from __future__ import annotations
import json, os, subprocess, sys, time
from pathlib import Path


def _curl(url: str, connect: str='2', max_t: str='5') -> tuple[int,str]:
    p = subprocess.run(['curl','--connect-timeout',connect,'--max-time',max_t,'-fsS',url], capture_output=True, text=True)
    return p.returncode, p.stdout


def main() -> int:
    root = Path.cwd()
    base_url = os.environ.get('ATLAS_E2E_BASE_URL', 'http://127.0.0.1:18080')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    release = os.environ.get('ATLAS_E2E_RELEASE_NAME', 'atlas-e2e')
    local_port = os.environ.get('ATLAS_E2E_LOCAL_PORT', '18080')
    connect_t = os.environ.get('ATLAS_SMOKE_CONNECT_TIMEOUT_SECS', '2')
    max_t = os.environ.get('ATLAS_SMOKE_MAX_TIME_SECS', '5')
    retries = int(os.environ.get('ATLAS_SMOKE_HEALTH_RETRIES', '20'))
    query_retries = int(os.environ.get('ATLAS_SMOKE_QUERY_RETRIES', '3'))
    smoke_dir = root / 'artifacts/ops/e2e/smoke'
    smoke_dir.mkdir(parents=True, exist_ok=True)
    out = smoke_dir/'requests.log'; out.write_text('', encoding='utf-8')
    resp_json = smoke_dir/'responses.jsonl'; resp_json.write_text('', encoding='utf-8')
    lockfile = root/'ops/e2e/smoke/queries.lock'
    golden_status = root/'ops/e2e/smoke/goldens/status_codes.json'
    run_id = os.environ.get('RUN_ID', os.environ.get('OPS_RUN_ID','local'))
    pf = None
    rc, _ = _curl(f'{base_url}/healthz', connect_t, max_t)
    if rc != 0:
        pod = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pods -l app.kubernetes.io/instance='{release}' --field-selector=status.phase=Running -o name | tail -n1 | cut -d/ -f2"], text=True).strip()
        pf_log = smoke_dir/'port-forward.log'
        pf = subprocess.Popen(['kubectl','-n',ns,'port-forward',f'pod/{pod}',f'{local_port}:18080'], stdout=pf_log.open('wb'), stderr=subprocess.STDOUT)
        base_url = f'http://127.0.0.1:{local_port}'
        for _ in range(retries):
            rc, _ = _curl(f'{base_url}/healthz', connect_t, max_t)
            if rc == 0: break
            time.sleep(1)
        if rc != 0:
            print(f'smoke failed: service not healthy after port-forward retries (base_url={base_url})', file=sys.stderr)
            return 1
    try:
        _, datasets_body = _curl(f'{base_url}/v1/datasets', connect_t, max_t)
        has_datasets = '"dataset"' in datasets_body
        golden = json.loads(golden_status.read_text(encoding='utf-8')) if golden_status.exists() else {}
        for q in [ln.strip() for ln in lockfile.read_text(encoding='utf-8').splitlines() if ln.strip()]:
            if q.startswith(('/v1/genes','/v1/transcripts','/v1/diff/','/v1/sequence/','/v1/releases/','/v1/datasets/')) and not has_datasets:
                continue
            status = ''
            body = ''
            for _ in range(query_retries):
                tmp = smoke_dir/'body.tmp'
                p = subprocess.run(['curl','--connect-timeout',connect_t,'--max-time',max_t,'-sS','-o',str(tmp),'-w','%{http_code}',f'{base_url}{q}'], capture_output=True, text=True)
                status = (p.stdout or '').strip()
                body = tmp.read_text(encoding='utf-8', errors='replace') if tmp.exists() else ''
                tmp.unlink(missing_ok=True)
                if status and status != '000':
                    break
                time.sleep(1)
            expected = str(golden.get(q,'')) if q in golden else ''
            if expected and status != expected:
                print(f'status mismatch for {q} expected={expected} got={status}', file=sys.stderr); return 1
            if q == '/metrics':
                if 'bijux_' not in body: return 1
            elif q in ('/healthz','/readyz'):
                if not body: return 1
            elif not body:
                return 1
            with resp_json.open('a', encoding='utf-8') as fh:
                fh.write(json.dumps({'run_id': run_id, 'path': q, 'status': int(status or '0')}) + '\n')
            with out.open('a', encoding='utf-8') as fh:
                fh.write(f'ok {q}\n')
                print(f'ok {q}')
        return 0
    finally:
        if pf is not None:
            pf.terminate()
            try: pf.wait(timeout=3)
            except Exception: pf.kill()

if __name__ == '__main__': raise SystemExit(main())
