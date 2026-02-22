from __future__ import annotations
import os, subprocess, sys, time

def _curl(url: str) -> str:
    return subprocess.check_output(['curl','--connect-timeout','2','--max-time','5','-fsS',url], text=True)

def main() -> int:
    base_url = os.environ.get('ATLAS_E2E_BASE_URL', 'http://127.0.0.1:18080')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    release = os.environ.get('ATLAS_E2E_RELEASE_NAME', 'atlas-e2e')
    local_port = os.environ.get('ATLAS_E2E_LOCAL_PORT', '18080')
    pf = None
    try:
        try:
            _curl(f'{base_url}/healthz')
        except Exception:
            if subprocess.call(['kubectl','config','current-context'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
                print('metrics runtime check skipped: kubectl context is not configured'); return 0
            pod = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pods -l app.kubernetes.io/instance='{release}' --field-selector=status.phase=Running -o name 2>/dev/null | tail -n1 | cut -d/ -f2"], text=True).strip()
            if not pod:
                print(f"metrics runtime check skipped: no running atlas pod found in namespace '{ns}'")
                return 0
            pf = subprocess.Popen(['kubectl','-n',ns,'port-forward',f'pod/{pod}',f'{local_port}:18080'], stdout=open('/tmp/atlas-metrics-port-forward.log','wb'), stderr=subprocess.STDOUT)
            base_url = f'http://127.0.0.1:{local_port}'
            for _ in range(10):
                try:
                    _curl(f'{base_url}/healthz'); break
                except Exception: time.sleep(1)
        metrics = _curl(f'{base_url}/metrics')
        datasets = subprocess.run(['curl','--connect-timeout','2','--max-time','5','-fsS',f'{base_url}/v1/datasets'], capture_output=True, text=True)
        has_datasets = '"dataset"' in (datasets.stdout or '')
        required = [
            'bijux_http_requests_total',
            'bijux_http_request_latency_p95_seconds',
            'bijux_overload_shedding_active',
            'bijux_store_breaker_open',
            'bijux_errors_total',
        ]
        if has_datasets:
            required += ['bijux_dataset_hits','bijux_dataset_misses','bijux_store_download_p95_seconds']
        for m in required:
            if not any(line.startswith(m) for line in metrics.splitlines()):
                print(f'missing metric: {m}', file=sys.stderr); return 1
        print('metrics verified')
        return 0
    finally:
        if pf is not None:
            pf.terminate()
            try: pf.wait(timeout=3)
            except Exception: pf.kill()

if __name__ == '__main__': raise SystemExit(main())
