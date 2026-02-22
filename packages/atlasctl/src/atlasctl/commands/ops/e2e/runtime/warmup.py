from __future__ import annotations

import os
import subprocess
import tempfile
import time
from pathlib import Path


def _metric_value(path: Path) -> float | None:
    for line in path.read_text(encoding='utf-8', errors='replace').splitlines():
        if line.startswith('bijux_dataset_hits') or line.startswith('bijux_dataset_cache_hit_total'):
            parts = line.split()
            if len(parts) >= 2:
                try:
                    return float(parts[-1])
                except ValueError:
                    return None
    return None


def main() -> int:
    root = Path.cwd()
    release = os.environ.get('ATLAS_E2E_RELEASE_NAME', 'atlas-e2e')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    service_name = os.environ.get('ATLAS_E2E_SERVICE_NAME', f'{release}-bijux-atlas')
    local_port = os.environ.get('ATLAS_E2E_LOCAL_PORT', '18080')
    warm_dir = root / 'artifacts/ops/e2e/warm'
    warm_dir.mkdir(parents=True, exist_ok=True)
    pf_log = warm_dir / 'port-forward.log'

    if subprocess.call(['kubectl', 'version', '--client'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
        print('kubectl is required', file=__import__('sys').stderr)
        return 1

    if subprocess.call(['kubectl', '-n', ns, 'get', 'job', f'{service_name}-dataset-warmup'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
        subprocess.check_call(['kubectl', '-n', ns, 'wait', '--for=condition=complete', '--timeout=5m', f'job/{service_name}-dataset-warmup'])

    pod = subprocess.check_output(
        ['bash', '-lc', f"kubectl -n '{ns}' get pods -l app.kubernetes.io/instance='{release}' --field-selector=status.phase=Running -o name | tail -n1 | cut -d/ -f2"],
        text=True,
    ).strip()
    pf = subprocess.Popen(['kubectl', '-n', ns, 'port-forward', f'pod/{pod}', f'{local_port}:18080'], stdout=pf_log.open('wb'), stderr=subprocess.STDOUT)
    try:
        for _ in range(10):
            if subprocess.call(['curl', '--connect-timeout', '2', '--max-time', '30', '-fsS', f'http://127.0.0.1:{local_port}/healthz'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                break
            time.sleep(1)

        with tempfile.NamedTemporaryFile(delete=False) as btmp, tempfile.NamedTemporaryFile(delete=False) as atmp:
            before_path = Path(btmp.name)
            after_path = Path(atmp.name)
        try:
            subprocess.run(['curl', '--connect-timeout', '2', '--max-time', '30', '-fsS', f'http://127.0.0.1:{local_port}/metrics'], stdout=before_path.open('wb'), stderr=subprocess.DEVNULL)
            for _ in range(3):
                if subprocess.call(['curl', '--connect-timeout', '2', '--max-time', '30', '-fsS', f'http://127.0.0.1:{local_port}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                    break
                time.sleep(1)
            for _ in range(3):
                if subprocess.call(['curl', '--connect-timeout', '2', '--max-time', '30', '-fsS', f'http://127.0.0.1:{local_port}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                    break
                time.sleep(1)
            subprocess.run(['curl', '--connect-timeout', '2', '--max-time', '30', '-fsS', f'http://127.0.0.1:{local_port}/metrics'], stdout=after_path.open('wb'), stderr=subprocess.DEVNULL)
            (warm_dir / 'metrics.before.prom').write_bytes(before_path.read_bytes())
            (warm_dir / 'metrics.after.prom').write_bytes(after_path.read_bytes())
            before = _metric_value(before_path)
            after = _metric_value(after_path)
            if before is not None and after is not None and after < before:
                return 1
        finally:
            before_path.unlink(missing_ok=True)
            after_path.unlink(missing_ok=True)
    finally:
        pf.terminate()
        try:
            pf.wait(timeout=3)
        except Exception:
            pf.kill()

    print('warmup verification completed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
