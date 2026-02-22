from __future__ import annotations

import os
import subprocess
from pathlib import Path


def main() -> int:
    profile = os.environ.get('ATLAS_OBS_PROFILE', 'kind')
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    if len(os.sys.argv) > 2 and os.sys.argv[1] == '--profile':
        profile = os.sys.argv[2]
    for endpoint in (
        f'{base}/healthz',
        f'{base}/v1/version',
        f'{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=3',
    ):
        subprocess.check_call(['curl', '-fsS', endpoint], stdout=subprocess.DEVNULL)
    out_dir = 'artifacts/ops/obs'
    subprocess.check_call(['bash', 'ops/obs/scripts/snapshot_metrics.sh', out_dir])
    subprocess.check_call(['bash', 'ops/obs/scripts/snapshot_traces.sh', out_dir])
    root = Path.cwd()
    if not (root / out_dir / 'metrics.prom').is_file() or (root / out_dir / 'metrics.prom').stat().st_size == 0:
        return 1
    if profile != 'local-compose':
        p = root / out_dir / 'traces.snapshot.log'
        if not p.is_file() or p.stat().st_size == 0:
            return 1
    print(f'observability pack smoke passed (profile={profile})')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
