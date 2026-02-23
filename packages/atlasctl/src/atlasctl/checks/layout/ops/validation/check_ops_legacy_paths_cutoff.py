#!/usr/bin/env python3
from __future__ import annotations
import re, sys
from datetime import date
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
DOC = ROOT / 'ops/docs/migration-window.md'
LEGACY = [
    ROOT/'ops/_schemas', ROOT/'ops/_meta/ownership.json', ROOT/'ops/_meta/surface.json', ROOT/'ops/_meta/contracts.json', ROOT/'ops/_meta/layer-contract.json', ROOT/'ops/registry/pins.json', ROOT/'ops/stack/version-manifest.json', ROOT/'ops/stack/versions.json'
]

def main() -> int:
    text = DOC.read_text(encoding='utf-8') if DOC.exists() else ''
    m = re.search(r'Cutoff .*: (\d{4}-\d{2}-\d{2})', text)
    if not m:
        print('missing cutoff date in ops/docs/migration-window.md', file=sys.stderr); return 1
    cutoff = date.fromisoformat(m.group(1))
    today = date.today()
    existing = [p.relative_to(ROOT).as_posix() for p in LEGACY if p.exists()]
    if today > cutoff and existing:
        print('legacy ops paths forbidden after cutoff:', file=sys.stderr)
        for rel in existing: print(f'- {rel}', file=sys.stderr)
        return 1
    print(f'ops legacy cutoff guard passed (today={today.isoformat()} cutoff={cutoff.isoformat()})')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
