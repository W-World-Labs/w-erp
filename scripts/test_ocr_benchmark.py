import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import ocr_benchmark as benchmark


class BenchmarkTests(unittest.TestCase):
    def case(self, **values):
        case = {'id': 'invoice', 'category': 'invoice', 'language': 'zh', 'split': 'test',
                'file': 'invoice.png', 'storage': 'repository', 'sha256': benchmark.digest(b'image'),
                'source': dict.fromkeys(('url', 'download_url', 'license', 'license_url', 'retrieved_at'), 'test'),
                'reference_text': None, 'expected_fields': {'supplier': 'seller', 'buyer': 'buyer'}}
        return dict(case, **values)

    def test_fields_association_and_types(self):
        self.assertFalse(benchmark.field_equal('seller', 'buyer'))
        self.assertTrue(benchmark.field_equal('13.00', 13))
        self.assertFalse(benchmark.field_equal('2026-09-07', '2026年09月07日'))
        self.assertFalse(benchmark.field_equal('2026-09-07', '2026-9-7'))
        self.assertTrue(benchmark.field_equal(None, None))
        self.assertFalse(benchmark.field_equal(None, 'unknown'))

    def test_number_boundaries(self):
        for text in ('113.00元', '13%', 'A13', '2026-09-13', '0.13'):
            self.assertFalse(benchmark.covered('13', text), text)
        self.assertTrue(benchmark.covered('13.00', '税额13元'))
        self.assertTrue(benchmark.covered('3000', '金额3,000.00元'))

    def test_text_dates_and_prompt_does_not_leak_answers(self):
        self.assertTrue(benchmark.covered('2019-02-19', '02/19/2019'))
        self.assertTrue(benchmark.covered('2018-08-28', '28-08-2018'))
        self.assertTrue(benchmark.covered('2019-02-19', '2019年2月19日'))
        self.assertFalse(benchmark.covered('2019-02-03', '02/03/2019'))
        prompt = benchmark.prompt_for('glm', 'fields', {'supplier': 'secret-target-value'})
        self.assertIn('supplier', prompt)
        self.assertNotIn('secret-target-value', prompt)

    def test_decimal_comma_percent_and_identifier_regressions(self):
        self.assertFalse(benchmark.covered('34.00', 'TOTAL 134,34'))
        self.assertFalse(benchmark.covered('13%', 'tax 113%'))
        self.assertFalse(benchmark.covered('0.05%', 'rate 10.05%'))
        self.assertTrue(benchmark.covered('34.00', 'TOTAL 34,00'))
        self.assertTrue(benchmark.covered('34,00', 'TOTAL 34.00'))
        self.assertTrue(benchmark.covered('13%', 'tax 13.00%'))
        self.assertTrue(benchmark.covered('0.05%', 'rate 0,05%'))
        self.assertFalse(benchmark.covered('13', 'tax 13 %'))
        self.assertFalse(benchmark.field_equal('142750', '0142750', 'invoice_number'))
        self.assertFalse(benchmark.field_equal('142750', '0142750', 'id'))
        self.assertFalse(benchmark.field_equal('142750', '0142750', 'contract_code'))
        self.assertTrue(benchmark.field_equal('34.00', 34, 'total_amount'))
        self.assertFalse(benchmark.field_equal('142750', '0142750', 'phone'))
        self.assertFalse(benchmark.field_equal('142750', 142750, 'reference'))
        self.assertTrue(benchmark.field_equal('15', 15, 'notice_days'))

    def test_numeric_trailing_sentence_punctuation(self):
        self.assertTrue(benchmark.covered('10', '10. Financial'))
        self.assertTrue(benchmark.covered('10', 'section 10, next'))
        self.assertFalse(benchmark.covered('10', 'value 10.5'))
        self.assertFalse(benchmark.covered('34', 'TOTAL 134,34'))
        self.assertFalse(benchmark.covered('34', 'TOTAL 134,34. Next'))
        self.assertTrue(benchmark.covered('34', 'TOTAL 34,00. Next'))

    def test_observed_text_keeps_protocol_failures_and_missing_denominator(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = self.case(reference_text='seller buyer')
            row = {'task': 'text', 'model': 'model', 'case_id': case['id'], 'input_sha256': case['sha256'], 'content': 'seller buyer', 'error': 'incomplete', 'raw_response': {'done': False}}
            benchmark.write_json(root / 'text/model/invoice.json', row)
            group = benchmark.score([case], root)['groups'][0]
            self.assertEqual(group['failed_or_missing'], 1)
            self.assertEqual(group['text_value_coverage'], 0)
            self.assertEqual(group['cer'], 1)
            self.assertEqual(group['observed_cer'], 0)
            self.assertEqual(group['observed_text_value_coverage'], 1)
            self.assertEqual(group['protocol_completion_unconfirmed'], 1)
            self.assertEqual(group['cases'][0]['observed_cer'], 0)
            group = benchmark.score([case, self.case(id='missing', reference_text='seller buyer')], root)['groups'][0]
            self.assertEqual(group['failed_or_missing'], 2)
            self.assertEqual(group['expected_values'], 4)
            self.assertEqual(group['observed_text_value_coverage'], .5)
            self.assertEqual(group['observed_cer'], .5)
            row['input_sha256'] = 'wrong'
            benchmark.write_json(root / 'text/model/invoice.json', row)
            group = benchmark.score([case], root)['groups'][0]
            self.assertEqual(group['observed_text_value_coverage'], 0)
            self.assertEqual(group['observed_cer'], 1)

    def test_paths_hashes_and_missing_cache(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = root / 'manifest.json'
            case = self.case()
            (root / 'invoice.png').write_bytes(b'image')
            manifest.write_text(json.dumps({'version': 1, 'cases': [case]}))
            self.assertEqual(benchmark.load_manifest(manifest, root), [case])
            self.assertEqual(benchmark.validate([case], manifest, root)[0]['status'], 'verified')
            self.assertEqual(benchmark.validate([self.case(storage='cache', file='absent')], manifest, root)[0]['status'], 'cache_missing_unverified')
            for file in ('../escape', '/tmp/escape'):
                with self.assertRaises(ValueError):
                    benchmark.case_path(self.case(file=file), manifest, root)
            (root / 'link').symlink_to('/tmp')
            with self.assertRaises(ValueError):
                benchmark.case_path(self.case(file='link/file'), manifest, root)
            manifest.write_text(json.dumps({'version': 1, 'cases': [case, case]}))
            with self.assertRaises(ValueError):
                benchmark.load_manifest(manifest, root)

    def test_score_preserves_denominators_and_no_annotation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = self.case()
            missing = self.case(id='missing', reference_text='abc')
            row = {'task': 'fields', 'model': 'model', 'case_id': 'invoice', 'input_sha256': case['sha256'], 'content': '{"supplier":"buyer","buyer":"seller"}', 'error': None}
            benchmark.write_json(root / 'fields/model/invoice.json', row)
            score = benchmark.score([case, missing], root)['groups'][0]
            self.assertEqual(score['expected_values'], 4)
            self.assertEqual(score['matched'], 0)
            self.assertEqual(score['failed_or_missing'], 1)
            self.assertIsNone(score['cer'])
            row.update(task='text', content='seller buyer')
            benchmark.write_json(root / 'text/model/invoice.json', row)
            text = next(g for g in benchmark.score([case, missing], root)['groups'] if g['task'] == 'text')
            self.assertEqual(text['cer'], 1)
            self.assertIsNone(text['cases'][0]['cer'])
            self.assertEqual(text['expected_values'], 4)
            self.assertEqual(text['matched'], 2)
            row.update(error='timeout')
            benchmark.write_json(root / 'text/model/invoice.json', row)
            text = next(g for g in benchmark.score([case, missing], root)['groups'] if g['task'] == 'text')
            self.assertEqual(text['failed_or_missing'], 2)
            self.assertEqual(text['matched'], 0)

    def test_visible_text(self):
        self.assertEqual(benchmark.normalized('<table><tr><td>甲</td><td>13</td></tr></table>'), '甲13')
        self.assertEqual(benchmark.normalized('甲&amp;乙'), '甲&乙')
        self.assertEqual(benchmark.normalized('<account>甲</account>'), '<account>甲</account>')

    def test_optional_text_targets_and_null_fields(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = self.case(expected_fields={'currency': 'USD', 'unknown': None}, expected_text_values={'currency': '$'})
            row = {'task': 'text', 'model': 'model', 'case_id': case['id'], 'input_sha256': case['sha256'], 'content': '$13', 'error': None}
            benchmark.write_json(root / 'text/model/invoice.json', row)
            group = benchmark.score([case], root)['groups'][0]
            self.assertEqual((group['matched'], group['expected_values']), (1, 1))
            row.update(task='fields', content='{"currency":"USD"}')
            benchmark.write_json(root / 'fields/model/invoice.json', row)
            group = next(g for g in benchmark.score([case], root)['groups'] if g['task'] == 'fields')
            self.assertEqual((group['matched'], group['expected_values']), (1, 2))

    def test_bad_download_hash_never_published(self):
        class Response:
            def __enter__(self):
                self.chunks = iter([b'wrong', b''])
                return self
            def __exit__(self, *args):
                pass
            def read1(self, size):
                return next(self.chunks)
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = self.case(storage='cache')
            case['source']['download_url'] = 'https://example.test/image.png'
            with patch.object(benchmark.urllib.request, 'build_opener') as opener:
                opener.return_value.open.side_effect = lambda *a, **kw: Response()
                rows = benchmark.fetch([case], root / 'manifest.json', root)
                self.assertEqual(opener.return_value.open.call_count, 2)
            self.assertEqual(rows[0]['status'], 'error')
            self.assertFalse((root / 'invoice.png').exists())

    def test_run_selection_metadata_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = self.case()
            manifest = root / 'manifest.json'
            manifest.write_text(json.dumps({'version': 1, 'cases': [case]}))
            with patch.object(benchmark, 'ollama', return_value={'version': 'test'}):
                benchmark.run([case], manifest, root, ['model'], root / 'out', 'fields')
                with self.assertRaises(ValueError):
                    benchmark.run([case], manifest, root, ['model'], root / 'out', 'fields')
            metadata_path = next((root / 'out').glob('*/*/_run.json'))
            metadata = json.loads(metadata_path.read_text())
            self.assertEqual(metadata['cases'], [case])
            self.assertEqual(metadata['ollama_version'], 'test')
            self.assertEqual(metadata['manifest_sha256'], benchmark.digest(manifest.read_bytes()))
            next((root / 'out').glob('*/*/invoice.json')).unlink()
            group = benchmark.score([case, self.case(id='excluded')], root / 'out')['groups'][0]
            self.assertEqual(group['expected_cases'], 1)
            self.assertEqual(group['failed_or_missing'], 1)
            self.assertEqual(group['excluded_case_ids'], ['excluded'])
            self.assertIsNone(group['mean_wall_seconds_observed'])
            self.assertEqual(len(group['errors']), 1)

    def test_run_records_failure_without_model_request(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'manifest.json').write_text(json.dumps({'version': 1, 'cases': [self.case()]}))
            with patch.object(benchmark, 'ollama', return_value={}) as request:
                benchmark.run([self.case()], root / 'manifest.json', root, ['glm-ocr:q8_0'], root / 'out', 'text')
            records = list((root / 'out').glob('*/*/invoice.json'))
            self.assertEqual(len(records), 1)
            self.assertTrue(json.loads(records[0].read_text())['error'])
            self.assertEqual(request.call_count, 3)  # Version plus releases, no inference.
            self.assertEqual(benchmark.score([self.case()], root / 'out')['groups'][0]['failed_or_missing'], 1)


if __name__ == '__main__':
    unittest.main()
