from __future__ import annotations
import os, subprocess, sys

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    procs=[]
    for _ in range(20):
        procs.append(subprocess.Popen(['curl','-fsS',f'{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL))
    for p in procs:
        if p.wait() != 0: return 1
    out = subprocess.check_output(['curl','-fsS',f'{base}/metrics'], text=True)
    if not any(k in out for k in ('bijux_dataset_singleflight','bijux_dataset_waiters','bijux_dataset_cache_misses_total')):
        print('expected cache stampede metrics missing', file=sys.stderr); return 1
    print('cache stampede drill passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
