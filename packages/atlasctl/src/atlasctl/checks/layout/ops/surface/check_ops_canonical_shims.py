#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
LEGACY = ('e2e', 'load', 'observability', 'charts', 'datasets', 'fixtures')


def main() -> int:
    errors = []
    for name in LEGACY:
        p = ROOT / name
        if p.exists() or p.is_symlink():
            errors.append(f'deprecated root alias exists: {name}')
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print('ops canonical layout check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
