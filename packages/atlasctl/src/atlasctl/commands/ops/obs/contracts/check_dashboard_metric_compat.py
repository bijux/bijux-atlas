#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: ensure dashboard metric queries reference metrics present in the metrics contract.
# stability: internal
# called-by: ops/obs/tests/test_contracts.sh
import json,re,sys
from pathlib import Path
def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


root = _repo_root()
contract=json.loads((root/'ops/obs/contract/metrics-contract.json').read_text(encoding='utf-8'))
allowed=set(contract.get('required_metric_specs',{}).keys())
dash=json.loads((root/'ops/obs/grafana/atlas-observability-dashboard.json').read_text(encoding='utf-8'))
exprs=[]
for p in dash.get('panels',[]):
    for t in p.get('targets',[]):
        e=t.get('expr')
        if isinstance(e,str):
            exprs.append(e)
missing=[]
for e in exprs:
    for m in re.findall(r'\b(?:bijux|atlas)_[a-zA-Z0-9_]+\b', e):
        if m not in allowed:
            missing.append((m,e))
if missing:
    print('dashboard references metrics missing from contract',file=sys.stderr)
    for m,e in missing[:20]:
        print(f'- {m}: {e}',file=sys.stderr)
    raise SystemExit(1)
print('dashboard metric compatibility passed')
