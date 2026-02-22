from __future__ import annotations
import os, subprocess, sys
from pathlib import Path


def main() -> int:
    base_url = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:8080')
    datasets = os.environ.get('DATASETS', os.environ.get('ATLAS_STARTUP_WARMUP', ''))
    if not datasets:
        print('usage: DATASETS=release/species/assembly[,..] make ops-warm-datasets', file=sys.stderr)
        return 2
    out = Path('artifacts/ops/e2e/warm-datasets')
    out.mkdir(parents=True, exist_ok=True)
    for ds in [x.strip() for x in datasets.split(',') if x.strip()]:
        try:
            rel, species, assembly = ds.split('/', 2)
        except ValueError:
            return 2
        url = f'{base_url}/v1/genes?release={rel}&species={species}&assembly={assembly}&gene_id=GENE1'
        target = out / f'{rel}_{species}_{assembly}.json'
        ok = False
        for _ in range(3):
            rc = subprocess.call(['curl', '-fsS', url], stdout=target.open('wb'), stderr=subprocess.DEVNULL)
            if rc == 0:
                ok = True
                break
            import time; time.sleep(1)
        if not ok:
            target.write_text('', encoding='utf-8')
        print(f'warmed {ds}')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
