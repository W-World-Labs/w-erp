#!/usr/bin/env python3
"""Offline corpus validation, bounded download, serial local Ollama OCR and scoring."""
import argparse
import base64
import datetime
from contextlib import contextmanager
from decimal import Decimal, InvalidOperation
import hashlib
from html.parser import HTMLParser
import json
import math
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time
import unicodedata
import urllib.error
import urllib.parse
import urllib.request

MAX_BYTES = 20 * 1024 * 1024
DEFAULT_MANIFEST = Path(__file__).resolve().parents[1] / 'tests/ocr/manifest.json'
OPTIONS = {'num_gpu': 0, 'num_thread': 16, 'num_predict': 2048, 'temperature': 0, 'num_ctx': 8192, 'seed': 42}


@contextmanager
def deadline(seconds):
    """Bound the entire blocking operation, including DNS and slow streaming (POSIX CLI)."""
    def expired(signum, frame):
        raise TimeoutError(f'operation exceeded {seconds} seconds')
    previous = signal.signal(signal.SIGALRM, expired)
    signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile('w', dir=path.parent, delete=False, encoding='utf-8') as f:
        json.dump(value, f, ensure_ascii=False, indent=2, allow_nan=False)
        f.write('\n')
    Path(f.name).replace(path)


def case_path(case, manifest, cache):
    root = (manifest.parent if case['storage'] == 'repository' else cache).resolve()
    relative = Path(case['file'])
    path = (root / relative).resolve()
    if relative.is_absolute() or '..' in relative.parts or not path.is_relative_to(root) or path == root:
        raise ValueError(f"unsafe path: {case['id']}")
    return path


def load_manifest(manifest, cache):
    data = json.loads(manifest.read_text())
    if data.get('version') != 1 or not isinstance(data.get('cases'), list) or not data['cases']:
        raise ValueError('manifest requires version 1 and nonempty cases')
    ids = set()
    for case in data['cases']:
        for key in ('id', 'category', 'language', 'split', 'file', 'storage', 'sha256'):
            if not isinstance(case.get(key), str) or not case[key]:
                raise ValueError(f'missing/non-string {key}')
        if not re.fullmatch(r'[A-Za-z0-9_-]+', case['id']) or case['id'] in ids or case['id'] == '_run':
            raise ValueError('invalid/duplicate case id')
        ids.add(case['id'])
        if case['storage'] not in ('repository', 'cache') or not re.fullmatch(r'[a-fA-F0-9]{64}', case['sha256']):
            raise ValueError(f"invalid storage/hash: {case['id']}")
        source = case.get('source')
        if not isinstance(source, dict) or any(not isinstance(source.get(k), str) or not source[k] for k in ('url', 'download_url', 'license', 'license_url', 'retrieved_at')):
            raise ValueError(f"missing provenance: {case['id']}")
        if case.get('reference_text') is not None and not isinstance(case['reference_text'], str):
            raise ValueError('reference_text must be string/null')
        targets = case.get('expected_text_values', {})
        if not isinstance(targets, dict) or any(not isinstance(k, str) or isinstance(v, (dict, list)) for k, v in targets.items()):
            raise ValueError('expected_text_values must contain scalar values')
        fields = case.get('expected_fields')
        if not isinstance(fields, dict) or any(not isinstance(k, str) or isinstance(v, (dict, list)) for k, v in fields.items()):
            raise ValueError('expected_fields must contain scalar values')
        if any(isinstance(v, float) and not math.isfinite(v) for v in fields.values()):
            raise ValueError('nonfinite expected number')
        if Path(case['file']).suffix.lower() == '.pdf':
            if type(case.get('page')) is not int or case['page'] < 1:
                raise ValueError('PDF requires positive page')
        elif 'page' in case:
            raise ValueError('page only supported for PDF')
        case_path(case, manifest, cache)
    return data['cases']


def validate(cases, manifest, cache):
    rows = []
    for case in cases:
        path = case_path(case, manifest, cache)
        status = ('verified' if digest(path.read_bytes()) == case['sha256'].lower() else 'sha256_mismatch') if path.is_file() else ('cache_missing_unverified' if case['storage'] == 'cache' else 'repository_missing')
        rows.append({'id': case['id'], 'status': status})
    return rows


class HTTPSRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        if urllib.parse.urlsplit(newurl).scheme != 'https':
            raise ValueError('non-HTTPS redirect refused')
        return super().redirect_request(request, fp, code, msg, headers, newurl)


def fetch(cases, manifest, cache):
    results = []
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), HTTPSRedirects())
    for case in cases:
        if case['storage'] != 'cache':
            continue
        path = case_path(case, manifest, cache)
        if path.exists():
            results.append({'id': case['id'], 'status': 'verified' if digest(path.read_bytes()) == case['sha256'].lower() else 'sha256_mismatch'})
            continue
        error = None
        for attempt in range(2):
            try:
                url = case['source']['download_url']
                if urllib.parse.urlsplit(url).scheme != 'https':
                    raise ValueError('downloads require HTTPS')
                started = time.monotonic()
                with deadline(30), opener.open(url, timeout=30) as response:
                    body = bytearray()
                    while True:
                        if time.monotonic() - started > 30:
                            raise TimeoutError('download exceeded 30 seconds')
                        chunk = response.read1(min(65536, MAX_BYTES + 1 - len(body)))
                        if not chunk:
                            break
                        body.extend(chunk)
                        if len(body) > MAX_BYTES:
                            raise ValueError('download exceeds 20 MB')
                if digest(body) != case['sha256'].lower():
                    raise ValueError('download SHA256 mismatch')
                path.parent.mkdir(parents=True, exist_ok=True)
                with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as f:
                    f.write(body)
                Path(f.name).replace(path)
                error = None
                break
            except (OSError, ValueError) as exc:
                error = str(exc)
        results.append({'id': case['id'], 'status': 'error' if error else 'verified', 'error': error})
    return results


def prompt_for(model, task, fields):
    if task == 'fields':
        return ('Extract only the requested fields from the document. Return one JSON object. '
                'Use the exact keys below; missing/unknown values must be null. '
                'supplier is seller/销方 and buyer is purchaser/购方; do not swap them. '
                'Dates must be YYYY-MM-DD. Monetary amounts must be JSON numbers without currency symbols. '
                'Do not guess. Requested keys: ' + json.dumps(list(fields), ensure_ascii=False))
    if 'glm' in model.lower():
        return 'Text Recognition:'
    if 'paddle' in model.lower():
        return 'OCR:'
    return 'Recognize all text in this image. Preserve the original reading order and text; use Markdown for layout. Do not explain or summarize.'


def ollama(path, payload=None, timeout=180):
    request = urllib.request.Request('http://127.0.0.1:11434' + path, data=json.dumps(payload).encode() if payload is not None else None, headers={'Content-Type': 'application/json'})
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPHandler())
    # No redirects: credentials/images must never leave the configured loopback service.
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None
    opener.add_handler(NoRedirect())
    with deadline(timeout), opener.open(request, timeout=timeout) as response:
        raw = response.read(MAX_BYTES + 1)
        if len(raw) > MAX_BYTES:
            raise ValueError('Ollama response exceeds 20 MB')
        return json.loads(raw)


def run(cases, manifest, cache, models, output, task):
    failures = 0
    if len(set(models)) != len(models):
        raise ValueError('duplicate models in run')
    for model in models:
        if (output / task / digest(model.encode())[:16]).exists():
            raise ValueError(f'refusing to overwrite existing {task}/{model} output; choose a new output directory')
    manifest_sha = digest(manifest.read_bytes())
    for model in models:
        model_id = digest(model.encode())[:16]
        model_dir = output / task / model_id
        model_dir.mkdir(parents=True, exist_ok=False)
        metadata = {'model': model, 'task': task, 'case_ids': [c['id'] for c in cases],
                    'cases': cases, 'manifest_sha256': manifest_sha, 'options': OPTIONS,
                    'think': False, 'keep_alive': 0, 'load_policy': 'cold: unload before run; keep_alive=0 after every case',
                    'ollama_version': None, 'setup_error': None}
        try:
            metadata['ollama_version'] = ollama('/api/version', timeout=10).get('version')
            ollama('/api/generate', {'model': model, 'keep_alive': 0}, timeout=10)
        except (OSError, ValueError) as exc:
            metadata['setup_error'] = str(exc)
        write_json(model_dir / '_run.json', metadata)
        try:
            for case in cases:
                print(f'START {task} {model} {case["id"]}', flush=True)
                prompt = prompt_for(model, task, case['expected_fields'])
                row = {'model': model, 'model_digest': None, 'case_id': case['id'], 'task': task,
                       'input_sha256': None, 'options': OPTIONS, 'think': False, 'keep_alive': 0, 'prompt': prompt,
                       'raw_response': None, 'content': '', 'error': None, 'wall_seconds': None}
                started = time.monotonic()
                try:
                    if metadata['setup_error']:
                        raise ValueError(metadata['setup_error'])
                    path = case_path(case, manifest, cache)
                    data = path.read_bytes()
                    row['input_sha256'] = digest(data)
                    if row['input_sha256'] != case['sha256'].lower():
                        raise ValueError('input SHA256 mismatch')
                    with tempfile.TemporaryDirectory() as temp:
                        if path.suffix.lower() == '.pdf':
                            prefix = str(Path(temp) / 'page')
                            subprocess.run(['pdftoppm', '-f', str(case['page']), '-l', str(case['page']), '-singlefile', '-scale-to', '1600', '-png', str(path), prefix], check=True, capture_output=True, timeout=30)
                            data = Path(prefix + '.png').read_bytes()
                        row['image_sha256'] = digest(data)
                        # /api/tags is GET; obtain the locally installed content digest before inference.
                        tags = ollama('/api/tags', timeout=10)
                        row['model_digest'] = next((m.get('digest') for m in tags.get('models', []) if m.get('name') == model or m.get('model') == model), None)
                        if not row['model_digest']:
                            raise ValueError('model not found in local catalog under exact name')
                        request = {'model': model, 'messages': [{'role': 'user', 'content': prompt, 'images': [base64.b64encode(data).decode()]}], 'stream': False, 'options': OPTIONS, 'think': False, 'keep_alive': 0}
                        if task == 'fields':
                            request['format'] = 'json'
                        response = ollama('/api/chat', request)
                        row['raw_response'] = response
                        row['content'] = response.get('message', {}).get('content', '')
                        if not isinstance(row['content'], str) or not row['content'].strip() or response.get('error') or not response.get('done') or response.get('done_reason') in ('length', 'max_tokens'):
                            raise ValueError('empty, incomplete, truncated or failed model response')
                        if task == 'fields' and not isinstance(json.loads(row['content']), dict):
                            raise ValueError('fields output must be JSON object')
                except (OSError, ValueError, KeyError, subprocess.SubprocessError) as exc:
                    row['error'] = str(exc)
                failures += int(row['error'] is not None)
                row['wall_seconds'] = time.monotonic() - started
                write_json(output / task / model_id / (case['id'] + '.json'), row)
                print(f'DONE {task} {model} {case["id"]} seconds={row["wall_seconds"]:.3f} error={row["error"]}', flush=True)
        finally:
            try:
                ollama('/api/generate', {'model': model, 'keep_alive': 0}, timeout=10)
            except (OSError, ValueError):
                pass
    return failures


class VisibleText(HTMLParser):
    layout = {'table', 'thead', 'tbody', 'tfoot', 'tr', 'td', 'th', 'caption', 'div', 'span', 'p', 'br', 'hr', 'h1', 'h2', 'h3', 'h4', 'ul', 'ol', 'li', 'pre', 'code', 'strong', 'em', 'b', 'i'}

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []

    def handle_starttag(self, tag, attrs):
        self.parts.append('\n' if tag in self.layout else self.get_starttag_text())

    def handle_endtag(self, tag):
        # Retain unknown markup, which may itself be document text.
        if tag in self.layout:
            self.parts.append('\n')
        else:
            self.parts.append(f'</{tag}>')

    def handle_data(self, data):
        self.parts.append(data)


def normalized(text):
    parser = VisibleText()
    parser.feed(str(text))
    parser.close()
    lines = []
    for line in ''.join(parser.parts).splitlines():
        if re.fullmatch(r'\s*(?:```.*|[|:\- ]{3,})\s*', line):
            continue
        line = re.sub(r'^\s*(?:#{1,6}\s+|[-*+]\s+|>\s*)', '', line)
        lines.append(re.sub(r'[|*_`~]', '', line))
    return ''.join(unicodedata.normalize('NFKC', '\n'.join(lines)).split())


def distance(a, b):
    previous = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        current = [i]
        for j, y in enumerate(b, 1):
            current.append(min(current[-1] + 1, previous[j] + 1, previous[j - 1] + (x != y)))
        previous = current
    return previous[-1]


def field_equal(expected, actual, key=""):
    if re.search(r"(?:^|_)(?:id|number|code|no)(?:$|_)", key.lower()):
        return type(actual) is type(expected) and actual == expected
    if expected is None:
        return actual is None
    if isinstance(expected, bool):
        return type(actual) is bool and actual == expected
    if re.fullmatch(r'\d{4}-\d{2}-\d{2}', str(expected)):
        try:
            return isinstance(actual, str) and datetime.date.fromisoformat(actual).isoformat() == actual == expected
        except ValueError:
            return False
    numeric_field = not key or bool(re.search(r'(?:^|_)(?:amount|quantity|qty|price|days|count|total|subtotal|balance|area|rate|cap)(?:$|_)', key.lower()))
    if numeric_field and re.fullmatch(r'[+-]?\d+(?:\.\d+)?', str(expected)):
        try:
            return not isinstance(actual, bool) and re.fullmatch(r'[+-]?\d+(?:\.\d+)?', str(actual)) is not None and Decimal(str(expected)) == Decimal(str(actual))
        except InvalidOperation:
            return False
    return isinstance(actual, str) and unicodedata.normalize('NFKC', actual).strip() == unicodedata.normalize('NFKC', str(expected)).strip()


def numeric_token(value):
    value = value.strip()
    percent = value.endswith('%')
    value = value[:-1].rstrip() if percent else value
    if re.fullmatch(r'[+-]?\d+(?:\.\d+)?', value):
        pass
    elif re.fullmatch(r'[+-]?\d{1,3}(?:,\d{3})+(?:\.\d+)?', value):
        value = value.replace(',', '')
    elif re.fullmatch(r'[+-]?\d+,\d{1,2}', value):
        value = value.replace(',', '.')
    elif re.fullmatch(r'[+-]?\d{1,3}(?:\.\d{3})+,\d{1,2}', value):
        value = value.replace('.', '').replace(',', '.')
    else:
        return None
    return Decimal(value), percent


def covered(value, content):
    if value is None:
        return False  # Absence cannot be established by value presence.
    text = unicodedata.normalize('NFKC', content)
    if re.fullmatch(r'\d{4}-\d{2}-\d{2}', str(value)):
        text = re.sub(r'(\d{4})[年/.-](\d{1,2})[月/.-](\d{1,2})日?', lambda m: f'{m[1]}-{int(m[2]):02}-{int(m[3]):02}', text)
        if re.search(r'(?<![0-9])' + re.escape(str(value)) + r'(?![0-9])', text):
            return True
        for match in re.finditer(r'(?<![0-9])(\d{1,2})[/-](\d{1,2})[/-](\d{4})(?![0-9])', text):
            a, b, year = map(int, match.groups())
            # Do not guess ambiguous day/month ordering.
            month, day = (a, b) if b > 12 else ((b, a) if a > 12 else (None, None))
            if month is not None:
                try:
                    if datetime.date(year, month, day).isoformat() == value:
                        return True
                except ValueError:
                    pass
        return False
    target = numeric_token(str(value))
    if target is not None:
        # Capture the entire punctuation-connected token before interpreting it.
        # This prevents matching 34 inside 134,34 or 13% inside 113%.
        tokens = re.finditer(r'(?<![A-Za-z0-9_.,+-])[+-]?\d[\d.,]*(?:\s*%)?(?![A-Za-z0-9_.,%+-])', text)
        return any(numeric_token(match.group().rstrip('.,')) == target for match in tokens)
    return bool(normalized(value)) and normalized(value) in normalized(text)


def score(cases, output):
    records = [json.loads(p.read_text()) for p in output.glob('*/*/*.json')]
    result = {'method': 'Visible-text CER removes HTML/Markdown layout, not a table-structure metric. Text value coverage is NOT field accuracy. Fields task compares each named field. Missing/error cases retain denominators. Unannotated text has no CER. Observed text metrics include returned content from protocol failures with valid input hashes; they are NOT acceptable completion or success rates. Missing/empty outputs retain denominators.', 'groups': []}
    for task, model in sorted({(r['task'], r['model']) for r in records}):
        rows = [r for r in records if r['task'] == task and r['model'] == model and 'case_id' in r]
        metadata = [r for r in records if r['task'] == task and r['model'] == model and 'case_id' not in r]
        if len(metadata) > 1:
            raise ValueError(f'duplicate run metadata: {task}/{model}')
        selected_ids = metadata[0]['case_ids'] if metadata else [c['id'] for c in cases]
        if len(set(selected_ids)) != len(selected_ids):
            raise ValueError('duplicate selected case ids')
        snapshots = metadata[0].get('cases', cases) if metadata else cases
        selected = [c for c in snapshots if c['id'] in selected_ids]
        if {c['id'] for c in selected} != set(selected_ids):
            raise ValueError('selected cases missing from run snapshot')
        group = {'task': task, 'model': model, 'excluded_case_ids': sorted({c['id'] for c in cases} - set(selected_ids)), 'expected_cases': len(selected), 'failed_or_missing': 0, 'matched': 0, 'expected_values': 0, 'cer_edits': 0, 'cer_characters': 0, 'cases': []}
        if task == 'text':
            group.update(observed_matched=0, observed_cer_edits=0, protocol_completion_unconfirmed=0)
        for case in selected:
            found = [r for r in rows if r['case_id'] == case['id']]
            row = found[0] if len(found) == 1 else None
            failed = row is None or bool(row.get('error')) or not row.get('content', '').strip()
            if row and row.get('input_sha256') != case['sha256'].lower():
                failed = True
            error = ('missing or duplicate case output' if row is None else row.get('error'))
            if failed and not error:
                error = 'empty output or input hash mismatch'
            content = '' if failed else row['content']
            observed = row.get('content', '') if row and row.get('input_sha256') == case['sha256'].lower() else ''
            if not isinstance(observed, str):
                observed = ''
            actual = {}
            if task == 'fields' and not failed:
                try:
                    actual = json.loads(content)
                    if not isinstance(actual, dict):
                        raise ValueError('not object')
                except ValueError as exc:
                    failed = True
                    error = str(exc)
            group['failed_or_missing'] += int(failed)
            fields = case['expected_fields']
            eligible = fields if task == 'fields' else {k: v for k, v in case.get('expected_text_values', fields).items() if v is not None}
            checks = {k: (not failed and (k in actual and field_equal(v, actual[k], k) if task == 'fields' else covered(v, content))) for k, v in eligible.items()}
            group['matched'] += sum(checks.values())
            group['expected_values'] += len(eligible)
            item = {'id': case['id'], 'failed_or_missing': failed, 'error': error, 'wall_seconds': row.get('wall_seconds') if row else None, 'field_matches' if task == 'fields' else 'value_coverage': checks, 'cer': None}
            if task == 'text':
                observed_checks = {k: bool(observed.strip()) and covered(v, observed) for k, v in eligible.items()}
                item['observed_value_coverage'] = observed_checks
                item['observed_cer'] = None
                raw = row.get('raw_response') if row else None
                item['protocol_completion_unconfirmed'] = bool(observed.strip()) and (not isinstance(raw, dict) or raw.get('done') is not True)
                group['protocol_completion_unconfirmed'] += int(item['protocol_completion_unconfirmed'])
                group['observed_matched'] += sum(observed_checks.values())
            reference = case.get('reference_text')
            if task == 'text' and reference and normalized(reference):
                a, b = normalized(reference), normalized(content)
                edits = distance(a, b)
                group['cer_characters'] += len(a)
                group['cer_edits'] += edits
                item['cer'] = edits / len(a)
                observed_edits = distance(a, normalized(observed))
                group['observed_cer_edits'] += observed_edits
                item['observed_cer'] = observed_edits / len(a)
            group['cases'].append(item)
        timings = [r['wall_seconds'] for r in group['cases'] if type(r['wall_seconds']) in (int, float) and math.isfinite(r['wall_seconds']) and r['wall_seconds'] >= 0]
        group['mean_wall_seconds_observed'] = sum(timings) / len(timings) if timings else None
        group['errors'] = [{'id': r['id'], 'error': r['error']} for r in group['cases'] if r['error']]
        group['cer'] = group['cer_edits'] / group['cer_characters'] if group['cer_characters'] else None
        group['field_accuracy' if task == 'fields' else 'text_value_coverage'] = group['matched'] / group['expected_values'] if group['expected_values'] else None
        if task == 'text':
            group['observed_text_value_coverage'] = group['observed_matched'] / group['expected_values'] if group['expected_values'] else None
            group['observed_cer'] = group['observed_cer_edits'] / group['cer_characters'] if group['cer_characters'] else None
        result['groups'].append(group)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['validate', 'fetch', 'run', 'score'])
    parser.add_argument('--manifest', type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument('--cache-dir', type=Path, default=Path.home() / '.cache/w-erp-ocr-corpus')
    parser.add_argument('--model', action='append')
    parser.add_argument('--case', action='append', dest='case_ids')
    parser.add_argument('--output-dir', type=Path)
    parser.add_argument('--task', choices=['text', 'fields'], default='text')
    args = parser.parse_args()
    try:
        cases = load_manifest(args.manifest, args.cache_dir)
    except (OSError, ValueError, TypeError, KeyError) as exc:
        parser.error(str(exc))
    if args.case_ids:
        unknown = set(args.case_ids) - {c['id'] for c in cases}
        if unknown:
            parser.error('unknown cases: ' + ', '.join(sorted(unknown)))
        cases = [c for c in cases if c['id'] in args.case_ids]
    if args.command in ('validate', 'fetch'):
        rows = globals()[args.command](cases, args.manifest, args.cache_dir)
        print(json.dumps(rows, ensure_ascii=False, indent=2))
        return int(any(r['status'] not in ('verified', 'cache_missing_unverified') for r in rows))
    if args.output_dir is None or (args.command == 'run' and not args.model):
        parser.error('run/score require --output-dir; run also requires --model')
    if args.command == 'run':
        return int(run(cases, args.manifest, args.cache_dir, args.model, args.output_dir, args.task) > 0)
    else:
        result = score(cases, args.output_dir)
        write_json(args.output_dir / 'scores.json', result)
        lines = ['# OCR benchmark', '', result['method'], '']
        for group in result['groups']:
            lines.append(f"- {group['model']} / {group['task']}: failed or missing {group['failed_or_missing']}/{group['expected_cases']}; matches {group['matched']}/{group['expected_values']}; CER {group['cer']}; mean seconds {group['mean_wall_seconds_observed']}")
            if group['task'] == 'text':
                lines.append(f"  Observed only (not accepted success): CER {group['observed_cer']}; value coverage {group['observed_text_value_coverage']}; completion unconfirmed {group['protocol_completion_unconfirmed']}")
            lines.append(f"  Excluded: {', '.join(group['excluded_case_ids']) or 'none'}; errors: {json.dumps(group['errors'], ensure_ascii=False)}")
        (args.output_dir / 'report.md').write_text('\n'.join(lines) + '\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
