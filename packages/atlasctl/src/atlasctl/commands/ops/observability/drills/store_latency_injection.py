from __future__ import annotations
import os, subprocess, sys, time
from pathlib import Path

def main() -> int:
    root = Path.cwd(); base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    subprocess.check_call([str(root/'stack/faults/inject.sh'),'toxiproxy-latency','1500','200'])
    time.sleep(3)
    subprocess.call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    out = subprocess.check_output(['curl','-fsS',f'{base}/metrics'], text=True)
    if 'bijux_store_breaker_open' not in out:
        print('store breaker metric missing under injected store latency', file=sys.stderr); return 1
    print('store latency injection drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
