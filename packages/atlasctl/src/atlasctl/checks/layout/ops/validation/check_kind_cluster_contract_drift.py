#!/usr/bin/env python3
from __future__ import annotations
import hashlib, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    contract = ROOT / 'ops/CONTRACT.md'
    text = contract.read_text(encoding='utf-8', errors='ignore') if contract.exists() else ''
    m = re.search(r'^\s*(?:-\s*)?kind-cluster-contract-hash:\s*`([a-f0-9]+)`\s*$', text, re.M)
    if not m:
        print('missing kind-cluster-contract-hash marker in ops/CONTRACT.md', file=sys.stderr)
        return 1
    marker = m.group(1)
    parts = sorted((ROOT / 'ops/stack/kind').glob('cluster*.yaml'))
    h = hashlib.sha256()
    for p in parts:
        h.update(p.read_bytes())
    calc = h.hexdigest()
    if marker != calc:
        print('kind cluster drift detected; update ops/CONTRACT.md marker to bump contract', file=sys.stderr)
        print(f'expected: {calc}', file=sys.stderr)
        print(f'found:    {marker}', file=sys.stderr)
        return 1
    print('kind cluster contract drift check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
