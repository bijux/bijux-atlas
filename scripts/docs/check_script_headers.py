#!/usr/bin/env python3
# Purpose: enforce script header contract and docs linkage for scripts under scripts/.
# Inputs: scripts/* files and docs/development/scripts/INDEX.md.
# Outputs: non-zero exit on missing headers or missing docs script-group references.
from __future__ import annotations
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
script_paths = sorted([p for p in (ROOT/'scripts').rglob('*') if p.is_file() and p.suffix in {'.sh','.py'}])
errors = []
for p in script_paths:
    if "/scripts/_internal/" in p.as_posix():
        continue
    txt = p.read_text(encoding='utf-8', errors='ignore').splitlines()
    head = '\n'.join(txt[:12])
    if p.suffix == '.sh' and not (
        head.startswith('#!/usr/bin/env sh')
        or head.startswith('#!/bin/sh')
        or head.startswith('#!/usr/bin/env bash')
        or head.startswith('#!/bin/bash')
        or head.startswith('#!/usr/bin/env python3')
    ):
        errors.append(f'{p}: missing shebang')
    if p.suffix == '.py' and not head.startswith('#!/usr/bin/env python3'):
        errors.append(f'{p}: missing shebang')
    legacy_header = 'Purpose:' in head and 'Inputs:' in head and 'Outputs:' in head
    modern_header = all(token in head.lower() for token in ('owner:', 'purpose:', 'stability:', 'called-by:'))
    if not (legacy_header or modern_header):
        errors.append(f'{p}: missing script header contract (Purpose/Inputs/Outputs or owner/purpose/stability/called-by)')
    rel = p.relative_to(ROOT).as_posix()
    if rel.startswith("scripts/public/"):
        required = ("owner:", "purpose:", "stability:", "called-by:")
        missing = [k for k in required if k not in head.lower()]
        if missing:
            errors.append(f"{p}: missing public header fields ({', '.join(missing)})")
    if p.as_posix().startswith(str((ROOT/'scripts'/'perf').as_posix())) and ('Owner:' not in head or 'Stability:' not in head):
        errors.append(f'{p}: missing extended header contract (Owner/Stability)')

idx = ROOT/'docs'/'development'/'scripts'/'INDEX.md'
if idx.exists():
    it = idx.read_text(encoding='utf-8')
    required_groups = ['scripts/contracts/', 'scripts/docs/', 'scripts/public/perf/', 'scripts/public/observability/', 'scripts/fixtures/', 'scripts/release/', 'scripts/layout/', 'scripts/bin/', 'scripts/public/', 'scripts/internal/', 'scripts/dev/', 'scripts/tools/']
    for group in required_groups:
        if group not in it:
            errors.append(f'{idx}: missing script group reference `{group}`')
else:
    errors.append(f'{idx}: missing scripts index')

if errors:
    print('script header check failed:', file=sys.stderr)
    for e in errors:
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)
print('script header check passed')
