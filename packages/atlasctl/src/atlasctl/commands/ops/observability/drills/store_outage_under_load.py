from __future__ import annotations
import os, subprocess, sys
from atlasctl.core.runtime.repo_root import find_repo_root
from pathlib import Path

def main() -> int:
    root = find_repo_root(); base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    subprocess.check_call([str(root/'stack/faults/inject.sh'),'block-minio','on'])
    try:
        subprocess.call(['./bin/atlasctl','ops','load','--report','text','run','--suite','store-outage-under-spike.json','--out','artifacts/perf/results'])
        subprocess.check_call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL)
        out = subprocess.check_output(['curl','-fsS',f'{base}/metrics'], text=True)
        if 'bijux_store_breaker_open' not in out:
            print('store breaker metric missing under load outage', file=sys.stderr); return 1
        print('store outage under load drill passed')
        return 0
    finally:
        subprocess.call([str(root/'stack/faults/inject.sh'),'block-minio','off'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
if __name__ == '__main__': raise SystemExit(main())
