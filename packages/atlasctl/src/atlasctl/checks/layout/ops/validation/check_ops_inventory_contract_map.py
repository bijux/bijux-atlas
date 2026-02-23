#!/usr/bin/env python3
from __future__ import annotations
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
MAP = ROOT / 'ops/inventory/contracts-map.json'
INV = ROOT / 'ops/inventory'

def main() -> int:
    payload = json.loads(MAP.read_text(encoding='utf-8'))
    rows = payload.get('items', []) if isinstance(payload, dict) else []
    mapped = {str(r.get('path','')) for r in rows if isinstance(r, dict)}
    errs: list[str] = []
    for p in sorted(INV.iterdir()):
        if not p.is_file():
            continue
        if p.name.startswith('.'):
            continue
        if p.suffix.lower() not in {'.json','.yaml','.yml'}:
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel not in mapped:
            errs.append(f'missing contracts-map entry: {rel}')
    for r in rows:
        if not isinstance(r, dict):
            continue
        rel = str(r.get('path',''))
        if rel and not (ROOT / rel).exists():
            errs.append(f'contracts-map target missing: {rel}')
    if errs:
        print('\n'.join(errs), file=sys.stderr)
        return 1
    print('ops inventory contract map check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
