#!/usr/bin/env python3
from __future__ import annotations
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
PAIRS = [
    ('ops/inventory/owners.json','ops/inventory/owners.json'),
    ('ops/inventory/surfaces.json','ops/inventory/surfaces.json'),
    ('ops/_meta/contracts.json','ops/inventory/contracts.json'),
    ('ops/_meta/layer-contract.json','ops/inventory/layers.json'),
]

def _norm_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))

def main() -> int:
    errs=[]
    for old,new in PAIRS:
        op,np = ROOT/old, ROOT/new
        if not op.exists() or not np.exists():
            continue
        try:
            if _norm_json(op) != _norm_json(np):
                errs.append(f'duplicate migration drift: {old} != {new}')
        except Exception as exc:
            errs.append(f'failed compare {old} vs {new}: {exc}')
    if errs:
        print('\n'.join(errs), file=sys.stderr)
        return 1
    print('ops migration duplicate inventory check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
