from __future__ import annotations
import os, subprocess
from pathlib import Path

def _errors_total(base: str) -> float:
    out = subprocess.check_output(['curl','-fsS',f'{base}/metrics'], text=True)
    total = 0.0
    for line in out.splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[0] == 'bijux_errors_total':
            try: total += float(parts[1])
            except ValueError: pass
    return total

def main() -> int:
    root = Path.cwd(); base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    alerts_file = root/'ops/obs/alerts/atlas-alert-rules.yaml'
    subprocess.check_call(['python3','packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_alerts_contract.py'])
    if subprocess.call(['bash','-lc','command -v promtool >/dev/null 2>&1']) == 0:
        subprocess.check_call(['promtool','check','rules',str(alerts_file)], stdout=subprocess.DEVNULL)
    before = _errors_total(base)
    subprocess.call(['curl','-fsS',f'{base}/v1/genes?release=bad&species=homo_sapiens&assembly=GRCh38&limit=1'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    import time; time.sleep(1)
    after = _errors_total(base)
    if after < before:
        return 1
    print('alert drill contract and signal assertions passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
