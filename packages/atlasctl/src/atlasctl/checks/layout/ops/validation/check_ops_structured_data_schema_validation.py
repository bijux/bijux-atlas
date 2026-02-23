#!/usr/bin/env python3
from __future__ import annotations
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
try:
    import yaml  # type: ignore
except Exception:
    yaml = None
try:
    import jsonschema  # type: ignore
except Exception:
    jsonschema = None
MAP = ROOT / 'ops/inventory/contracts-map.json'
ALLOW_UNSCHEMED = {'ops/inventory/contracts.json','ops/inventory/owner-docs.fragments.json'}

def _load_any(path: Path):
    if path.suffix.lower() == '.json':
        return json.loads(path.read_text(encoding='utf-8'))
    if yaml is None:
        raise RuntimeError('PyYAML unavailable')
    return yaml.safe_load(path.read_text(encoding='utf-8'))

def main() -> int:
    payload = json.loads(MAP.read_text(encoding='utf-8'))
    rows = [r for r in payload.get('items', []) if isinstance(r, dict)]
    errs=[]
    for row in rows:
        rel = str(row.get('path',''))
        schema_rel = str(row.get('schema','none'))
        if not rel:
            continue
        path = ROOT / rel
        if not path.exists():
            errs.append(f'missing inventory file: {rel}')
            continue
        try:
            data = _load_any(path)
        except Exception as exc:
            errs.append(f'failed parse {rel}: {exc}')
            continue
        if schema_rel == 'none' or rel in ALLOW_UNSCHEMED:
            continue
        schema_path = ROOT / schema_rel
        if not schema_path.exists():
            errs.append(f'missing schema for {rel}: {schema_rel}')
            continue
        if schema_path.suffix.lower() != '.json':
            continue
        if jsonschema is None:
            continue
        try:
            schema = json.loads(schema_path.read_text(encoding='utf-8'))
            if isinstance(schema, dict) and schema.get('$schema'):
                jsonschema.validate(data, schema)
        except Exception as exc:
            errs.append(f'schema validation failed {rel} via {schema_rel}: {exc}')
    if errs:
        print('\n'.join(errs), file=sys.stderr); return 1
    print('ops structured data schema validation passed'); return 0

if __name__ == '__main__':
    raise SystemExit(main())
