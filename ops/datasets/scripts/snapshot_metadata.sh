#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: write dataset metadata snapshot into ops artifacts.
# stability: public
# called-by: make ops-publish
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT="${OPS_RUN_DIR:-$ROOT/artifacts/ops/manual}/datasets"
mkdir -p "$OUT"
python3 - <<'PY'
from pathlib import Path
import json,hashlib,os
root=Path.cwd()
out=Path(os.environ.get('OPS_RUN_DIR', str(root/'artifacts/ops/manual')))/'datasets'
out.mkdir(parents=True, exist_ok=True)
manifest=root/'ops/datasets/manifest.lock'
qc=root/'artifacts/e2e-datasets/qc_report.json'
def sha(p:Path):
    return hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else 'missing'
meta={
  'manifest_lock_sha256': sha(manifest),
  'catalog_sha256': sha(root/'artifacts/e2e-datasets/catalog.json'),
  'qc_report_sha256': sha(qc),
}
(out/'metadata.snapshot.json').write_text(json.dumps(meta, indent=2)+'\n')
print(out/'metadata.snapshot.json')
PY
