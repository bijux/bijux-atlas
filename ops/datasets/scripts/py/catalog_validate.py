#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: validate catalog structure and deterministic merge ordering.
# stability: public
# called-by: make ops-catalog-validate
from __future__ import annotations
import json
from pathlib import Path
import sys
ROOT = Path(__file__).resolve().parents[4]
catalog = ROOT / 'artifacts/e2e-datasets/catalog.json'
if not catalog.exists():
    print('missing artifacts/e2e-datasets/catalog.json (run make ops-publish first)', file=sys.stderr)
    raise SystemExit(1)
data = json.loads(catalog.read_text())
if 'datasets' not in data or not isinstance(data['datasets'], list):
    print('catalog schema invalid: missing datasets[]', file=sys.stderr)
    raise SystemExit(1)
ids = []
for e in data['datasets']:
    ds=e.get('dataset',{})
    ids.append(f"{ds.get('release')}/{ds.get('species')}/{ds.get('assembly')}")
if ids != sorted(ids):
    print('catalog deterministic merge check failed: dataset ids not sorted', file=sys.stderr)
    raise SystemExit(1)
print('catalog validation passed')
