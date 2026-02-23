#!/usr/bin/env python3
from __future__ import annotations
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
POL = ROOT/'ops/inventory/generated-committed-mirror.json'
DIR = ROOT/'ops/_generated_committed'

def main() -> int:
    p = json.loads(POL.read_text(encoding='utf-8'))
    runtime = set(p.get('allow_runtime_compat', []))
    mirrors = {row['committed']: row.get('source','') for row in p.get('mirrors',[]) if isinstance(row, dict) and 'committed' in row}
    errs=[]
    for rel, src in mirrors.items():
        if not (ROOT/rel).exists(): errs.append(f'mirror target missing: {rel}')
        if src and not (ROOT/src).exists(): errs.append(f'mirror source missing: {src}')
    for fp in sorted(DIR.rglob('*')):
        if not fp.is_file(): continue
        rel = fp.relative_to(ROOT).as_posix()
        if rel in mirrors or rel in runtime: continue
        errs.append(f'unmapped committed generated file: {rel}')
    if errs:
        print('\n'.join(errs), file=sys.stderr); return 1
    print('ops generated committed mirror policy check passed'); return 0

if __name__ == '__main__':
    raise SystemExit(main())
