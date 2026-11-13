import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import contract_benchmark as b

class ContractBenchmarkTest(unittest.TestCase):
    def setUp(self):
        self.case=b.load()['cases'][0]

    def test_gold_and_negative_controls(self):
        data=b.load()
        self.assertEqual(len(data['cases']),18)
        for c in data['cases']:
            self.assertTrue(b.grade(c,{**c['expected'],'reason':'测试标准答案'})['strict'])
        for f in {c['family'] for c in data['cases']} - {'blank','missing'}:
            self.assertEqual({c['expected']['status'] for c in data['cases'] if c['family']==f},{'risk','protected'})

    def test_setup_failure_preserves_selected_cases(self):
        for failure in [[OSError('offline')],[{'version':'test'}, OSError('missing model')]]:
            with tempfile.TemporaryDirectory() as directory, patch.object(b,'ollama',side_effect=failure):
                out=Path(directory)/'run'
                result=b.run([self.case], 'missing-model', out)
                self.assertEqual(result['cases'],1)
                self.assertEqual(result['matched']['valid'],0)
                self.assertIn('setup_error',json.loads((out/'_run.json').read_text()))
                self.assertEqual(b.score(out)['matched']['strict'],0)

    def test_no_answer_leak(self):
        self.assertEqual(set(json.loads(b.prompt(self.case))),{'role','question','clauses'})

    def test_fabricated_and_duplicate_evidence(self):
        for evidence in [['C999'],['C1','C1']]:
            self.assertFalse(b.grade(self.case,{'status':'risk','evidence':evidence,'reason':'x'})['valid'])

    def test_wrong_label_and_missing_evidence(self):
        self.assertFalse(b.grade(self.case,{'status':'protected','evidence':['C1'],'reason':'x'})['strict'])
        r=b.grade(self.case,{'status':'risk','evidence':[],'reason':'x'})
        self.assertTrue(r['classification'])
        self.assertFalse(r['strict'])

    def test_malformed(self):
        for r in [None,[],{}, {'status':'risk','evidence':[{}],'reason':'x'}]:
            self.assertFalse(b.grade(self.case,r)['valid'])

    def test_missing_failed_and_tampered_stay_in_denominator(self):
        with tempfile.TemporaryDirectory() as directory:
            out=Path(directory)
            b.write_json(out/'_run.json',{'cases':[self.case]})
            self.assertEqual(b.score(out)['matched']['strict'],0)
            for error,sha in [('timeout',b.digest(b.prompt(self.case).encode())),(None,'wrong')]:
                b.write_json(out/(self.case['id']+'.json'),{'error':error,'case_sha256':sha,'parsed':{**self.case['expected'],'reason':'x'}})
                result=b.score(out)
                self.assertEqual(result['cases'],1)
                self.assertEqual(result['matched']['strict'],0)

if __name__=='__main__':
    unittest.main()
