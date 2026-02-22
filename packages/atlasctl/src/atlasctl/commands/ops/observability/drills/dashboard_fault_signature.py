from __future__ import annotations
import subprocess
from pathlib import Path

def main() -> int:
    root = Path.cwd(); f = root/'ops/obs/grafana/atlas-observability-dashboard.json'
    for key in ('shed rate','bulkhead saturation','cache hit ratio','store p95'):
        subprocess.check_call(['rg','-ni',key,str(f)], stdout=subprocess.DEVNULL)
    subprocess.check_call(['python3','packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_contract.py'])
    print('dashboard fault signature drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
