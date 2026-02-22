#!/usr/bin/env python3
from __future__ import annotations
import re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS_MK = ROOT / 'makefiles/ops.mk'


def main() -> int:
    if not OPS_MK.exists():
        print(f'missing {OPS_MK}', file=sys.stderr)
        return 1
    mk = OPS_MK.read_text(encoding='utf-8', errors='ignore')
    missing = []
    for script in sorted((ROOT / 'ops').rglob('scripts/*.sh')):
        rel = script.relative_to(ROOT).as_posix()
        if rel not in mk:
            missing.append(rel)
    if missing:
        for m in missing:
            print(f'ops script not mapped by make target: {m}', file=sys.stderr)
        return 1
    print('ops script coverage check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
