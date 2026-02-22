from __future__ import annotations
import os, subprocess
from pathlib import Path


def main() -> int:
    release = os.environ.get('ATLAS_DATASET_RELEASE', '110')
    species = os.environ.get('ATLAS_DATASET_SPECIES', 'homo_sapiens')
    assembly = os.environ.get('ATLAS_DATASET_ASSEMBLY', 'GRCh38')
    base = os.environ.get('ATLAS_E2E_BASE_URL', f"http://127.0.0.1:{os.environ.get('ATLAS_E2E_LOCAL_PORT','18080')}")
    out = Path('artifacts/ops/e2e/warm-shards')
    out.mkdir(parents=True, exist_ok=True)
    subprocess.check_call(['curl','-fsS',f'{base}/v1/genes?release={release}&species={species}&assembly={assembly}&region=chr1:1-100000&limit=10'], stdout=(out/'chr1.json').open('wb'))
    subprocess.check_call(['curl','-fsS',f'{base}/v1/genes?release={release}&species={species}&assembly={assembly}&region=chr2:1-100000&limit=10'], stdout=(out/'chr2.json').open('wb'))
    print('shard warmup completed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
