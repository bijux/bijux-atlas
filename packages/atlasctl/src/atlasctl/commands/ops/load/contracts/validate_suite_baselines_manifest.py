from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    cfg = ROOT / 'configs/ops/load/suite-baselines.json'
    schema = json.loads((ROOT / 'configs/ops/load/suite-baselines.schema.json').read_text(encoding='utf-8'))
    payload = json.loads(cfg.read_text(encoding='utf-8'))
    # lightweight validation without external dependency
    errs: list[str] = []
    if payload.get('schema_version') != schema['properties']['schema_version']['const']:
        errs.append('schema_version must be 1')
    if payload.get('kind') != schema['properties']['kind']['const']:
        errs.append('kind must be ops-load-suite-baselines')
    rows = payload.get('suites', [])
    if not isinstance(rows, list):
        errs.append('suites must be a list')
        rows = []
    seen: set[str] = set()
    manifest = json.loads((ROOT / 'ops/load/suites/suites.json').read_text(encoding='utf-8'))
    declared = {str(r.get('name', '')).strip() for r in manifest.get('suites', []) if isinstance(r, dict)}
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            errs.append(f'suites[{i}] must be object')
            continue
        name = str(row.get('name', '')).strip()
        if not name:
            errs.append(f'suites[{i}].name required')
            continue
        if name in seen:
            errs.append(f'duplicate suite baseline entry: {name}')
        seen.add(name)
        suite = str(row.get('suite', '')).strip()
        if suite not in declared:
            errs.append(f'{name}: suite `{suite}` not declared in ops/load/suites/suites.json')
        baseline_rel = str(row.get('baseline', '')).strip()
        if not baseline_rel:
            errs.append(f'{name}: baseline path required')
        else:
            p = ROOT / baseline_rel
            if not p.exists():
                errs.append(f'{name}: baseline file missing: {baseline_rel}')
        if row.get('mode') not in {'smoke', 'regression'}:
            errs.append(f'{name}: mode must be smoke|regression')
    if errs:
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print('ops load suite baselines manifest passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
