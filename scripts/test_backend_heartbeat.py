#!/usr/bin/env python3
"""Exercise the shipped server against a local Radar, using an isolated SQLite DB."""
import argparse
import http.server
import json
import os
import socket
import subprocess
import tempfile
import threading
import time
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('binary')
args = parser.parse_args()
binary = os.path.abspath(args.binary)
records = []


class Radar(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        architecture = any(e.get('type') == 'architecture_snapshot'
                           for e in payload.get('events', []))
        status = 503 if architecture and not any(r[2] for r in records) else 200
        records.append((self.path, payload, architecture, status, time.monotonic()))
        self.send_response(status)
        self.end_headers()
        self.wfile.write(b'{}')


server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Radar)
threading.Thread(target=server.serve_forever, daemon=True).start()
with socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    port = listener.getsockname()[1]
env = {k: v for k, v in os.environ.items()
       if not k.startswith(('W_RADAR', 'W_ID', 'DATABASE', 'SELF_UPDATE'))}
env.update(BIND_HOST='127.0.0.1', PORT=str(port), SELF_UPDATE_AUTO_APPLY='0',
           W_RADAR_CODE='test-code', W_RADAR_API_KEY='test-key',
           W_RADAR_ARCHITECTURE_API_KEY='test-key', W_RADAR_PROJECT_ID='w-erp',
           W_RADAR_INSTANCE_ID='erp-heartbeat-test',
           W_RADAR_ENDPOINT=f'http://127.0.0.1:{server.server_port}',
           HTTPS_PROXY='http://127.0.0.1:1', NO_PROXY='127.0.0.1,localhost')
try:
    with tempfile.TemporaryDirectory(prefix='erp-heartbeat-') as cwd:
        env['DATABASE_URL'] = f'sqlite:{cwd}/test.sqlite?mode=rwc'
        with open(f'{cwd}/server.log', 'w+') as log:
            process = subprocess.Popen([binary], cwd=cwd, env=env, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 45
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        log.seek(0)
                        raise AssertionError(log.read())
                    if sum(r[2] for r in records) >= 3:
                        break
                    time.sleep(.1)
                beats = [r for r in records if r[2]]
                caps = [r for r in records if r[0].endswith('/capabilities')]
                assert len(beats) >= 3, records
                assert [r[3] for r in beats[:3]] == [503, 200, 200]
                assert len(caps) == 1, caps
                payload = caps[0][1]
                assert payload['process']['pid'] == process.pid
                assert payload['project_id'] == 'w-erp'
                assert payload['instance_id'] == 'erp-heartbeat-test'
                assert payload['sdks']['xradar'] == '0.3.8'
                gaps = [beats[i + 1][4] - beats[i][4] for i in range(2)]
                assert all(8 < gap < 12 for gap in gaps), gaps
                for beat in beats:
                    snapshot = beat[1]['events'][0]['data']
                    assert snapshot['ttl_seconds'] == 30
                    assert snapshot['instance_id'] == 'erp-heartbeat-test'
                with urllib.request.urlopen(f'http://127.0.0.1:{port}/health', timeout=3) as response:
                    assert response.status == 200
            finally:
                process.terminate()
                process.wait(timeout=10)
            count = len(records)
            time.sleep(11)
            assert len(records) == count, 'heartbeat continued after shutdown'
            # A process that cannot own its listener must never claim to be alive.
            with socket.socket() as occupied:
                occupied.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                occupied.bind(('127.0.0.1', port))
                occupied.listen()
                failed = subprocess.run([binary], cwd=cwd, env=env, stdout=log,
                                        stderr=log, timeout=30)
                assert failed.returncode != 0
                time.sleep(.2)
                assert len(records) == count, 'heartbeat emitted before bind success'
            # Missing an architecture writer must not disable capabilities.
            env.pop('W_RADAR_API_KEY')
            env.pop('W_RADAR_ARCHITECTURE_API_KEY')
            records.clear()
            process = subprocess.Popen([binary], cwd=cwd, env=env, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 15
                while not records and time.monotonic() < deadline:
                    assert process.poll() is None
                    time.sleep(.1)
                capabilities = [r for r in records if r[0].endswith('/capabilities')]
                assert len(capabilities) == 1, records
                assert capabilities[0][1]['process']['pid'] == process.pid
                assert not any(r[2] for r in records)
            finally:
                process.terminate()
                process.wait(timeout=10)
            print(json.dumps(dict(capabilities=1, architecture=len(beats),
                                  recovery=[503, 200, 200], intervals=gaps,
                                  pid=process.pid, shutdown_stopped=True,
                                  bind_failure_silent=True, capabilities_without_writer=True)))
finally:
    server.shutdown()
