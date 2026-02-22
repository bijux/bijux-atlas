#!/usr/bin/env python3
from __future__ import annotations
import os, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
ALLOWLIST = ROOT / 'configs/ops/artifacts-allowlist.txt'


def main() -> int:
    if not ALLOWLIST.exists():
        print(f'missing allowlist: {ALLOWLIST}', file=sys.stderr)
        return 1
    artifacts = ROOT / 'artifacts'
    if not artifacts.exists():
        print('artifacts allowlist check passed (artifacts/ absent)')
        return 0
    if os.environ.get('ARTIFACTS_ALLOWLIST_STRICT', '0') != '1':
        print('artifacts allowlist check skipped (non-strict mode)')
        return 0
    allow = []
    for raw in ALLOWLIST.read_text(encoding='utf-8', errors='ignore').splitlines():
        s = raw.strip()
        if not s or s.startswith('#'):
            continue
        allow.append(s)
    errors = []
    if (artifacts / 'target').is_dir():
        errors.append('unexpected artifact directory: artifacts/target')
    for p in sorted(artifacts.rglob('*')):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith('artifacts/target/'):
            continue
        if not any(Path(rel).match(pat) for pat in allow):
            errors.append(f'unexpected artifact path: {rel}')
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print('artifacts allowlist check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
