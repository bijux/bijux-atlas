#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
DOC = ROOT / 'packages/atlasctl/docs/_generated/commands-inventory.md'


def main() -> int:
    if not DOC.exists():
        print(f'missing {DOC.relative_to(ROOT).as_posix()}')
        return 1
    text = DOC.read_text(encoding='utf-8', errors='ignore')
    errs: list[str] = []
    for token in ('`atlasctl ops`', '`atlasctl product`'):
        if token not in text:
            errs.append(f'commands-inventory missing {token}')
    if errs:
        print('\n'.join(errs))
        return 1
    print('commands inventory includes atlasctl ops/product groups')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
