from __future__ import annotations
import json, os, subprocess, sys
from pathlib import Path


def main() -> int:
    base_url = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:8080')
    top_n = int(os.environ.get('TOP_N', '5'))
    out = Path('artifacts/ops/e2e/warm-top')
    out.mkdir(parents=True, exist_ok=True)
    resp = out / 'datasets.json'
    subprocess.check_call(['curl', '-fsS', f'{base_url}/v1/datasets'], stdout=resp.open('wb'))
    obj = json.loads(resp.read_text(encoding='utf-8'))
    rows = []
    if isinstance(obj, dict):
        if isinstance(obj.get('datasets'), list):
            rows = obj['datasets']
        elif isinstance(obj.get('data'), list):
            rows = obj['data']
    items = []
    for d in rows[:top_n]:
        ds = d.get('dataset') if isinstance(d, dict) else None
        if isinstance(ds, dict):
            r, s, a = ds.get('release'), ds.get('species'), ds.get('assembly')
            if r and s and a:
                items.append(f'{r}/{s}/{a}')
    if not items:
        print('no datasets resolved from /v1/datasets', file=sys.stderr)
        return 1
    env = {**os.environ, 'DATASETS': ','.join(items)}
    return subprocess.call(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_datasets.py'], env=env)

if __name__ == '__main__':
    raise SystemExit(main())
