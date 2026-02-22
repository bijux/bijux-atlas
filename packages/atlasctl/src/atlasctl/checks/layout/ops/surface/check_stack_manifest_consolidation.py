#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    e2e = ROOT / 'ops/e2e'
    violations = sorted([p.relative_to(ROOT).as_posix() for p in e2e.rglob('*') if p.is_file() and p.suffix in {'.yaml', '.yml'}]) if e2e.exists() else []
    if violations:
        print('stack manifest consolidation check failed: manifests found under ops/e2e', file=sys.stderr)
        for v in violations:
            print(v, file=sys.stderr)
        return 1
    print('stack manifest consolidation check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
