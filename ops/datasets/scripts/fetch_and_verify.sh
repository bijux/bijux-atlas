#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: fetch dataset prerequisites and verify checksums from manifest lock.
# stability: public
# called-by: make ops-datasets-fetch
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
python3 "$ROOT/scripts/layout/check_dataset_manifest_lock.py"
if [ -f "$ROOT/ops/datasets/manifest.lock" ]; then
  :
else
  echo "missing ops/datasets/manifest.lock" >&2
  exit 1
fi
if [ ! -f "$ROOT/ops/fixtures/medium/v1/data/genes.gff3" ] || [ ! -f "$ROOT/ops/fixtures/medium/v1/data/genome.fa" ] || [ ! -f "$ROOT/ops/fixtures/medium/v1/data/genome.fa.fai" ]; then
  "$ROOT/scripts/fixtures/fetch-medium.sh" >/dev/null
fi
python3 - <<'PY'
from pathlib import Path
import json,hashlib,sys
root=Path.cwd()
lock=json.loads((root/'ops/datasets/manifest.lock').read_text())
manifest=json.loads((root/'ops/datasets/manifest.json').read_text())
name_to_ds={d['name']:d for d in manifest['datasets']}
errs=[]
for e in lock['entries']:
    ds=name_to_ds[e['name']]
    for key,expected in e.get('checksums',{}).items():
        rel=ds.get('paths',{}).get(key)
        if not rel or expected is None:
            continue
        p=root/rel
        got=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
        if got!=expected:
            errs.append(f"checksum mismatch {rel}: {got} != {expected}")
if errs:
    print("dataset checksum verification failed", file=sys.stderr)
    print("\n".join(errs), file=sys.stderr)
    raise SystemExit(1)
print("dataset checksum verification passed")
PY
"$ROOT/scripts/fixtures/fetch-real-datasets.sh" >/dev/null
