#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
ALLOWLIST = ROOT / 'configs/repo/root-files-allowlist.txt'


def main() -> int:
    if not ALLOWLIST.exists():
        print(f'missing allowlist: {ALLOWLIST}', file=sys.stderr)
        return 1
    allowed = sorted({ln.strip() for ln in ALLOWLIST.read_text(encoding='utf-8', errors='ignore').splitlines() if ln.strip() and not ln.strip().startswith('#')})
    allowed_set = set(allowed)
    fail = False
    actual = sorted(p.name for p in ROOT.iterdir() if p.is_file())
    for f in actual:
        if f not in allowed_set:
            print(f'unexpected root file: {f}', file=sys.stderr)
            fail = True
    if fail:
        print('update configs/repo/root-files-allowlist.txt if intentional', file=sys.stderr)
        return 1
    print('root dumping check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
