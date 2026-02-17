#!/usr/bin/env python3
# Purpose: enforce script header contract and docs linkage for scripts under scripts/.
# Inputs: scripts/* files and docs/development/scripts/INDEX.md.
# Outputs: non-zero exit on missing headers or missing docs script-group references.
from __future__ import annotations
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
script_paths = sorted([p for p in (ROOT/'scripts').rglob('*') if p.is_file() and p.suffix in {'.sh','.py'}])
errors = []
for p in script_paths:
    txt = p.read_text(encoding='utf-8', errors='ignore').splitlines()
    head = '\n'.join(txt[:12])
    if p.suffix == '.sh' and not (head.startswith('#!/usr/bin/env sh') or head.startswith('#!/bin/sh')):
        errors.append(f'{p}: missing shebang')
    if p.suffix == '.py' and not head.startswith('#!/usr/bin/env python3'):
        errors.append(f'{p}: missing shebang')
    if 'Purpose:' not in head or 'Inputs:' not in head or 'Outputs:' not in head:
        errors.append(f'{p}: missing script header contract (Purpose/Inputs/Outputs)')

idx = ROOT/'docs'/'development'/'scripts'/'INDEX.md'
if idx.exists():
    it = idx.read_text(encoding='utf-8')
    required_groups = ['scripts/contracts/', 'scripts/docs/', 'scripts/perf/', 'scripts/observability/', 'scripts/fixtures/', 'scripts/release/', 'scripts/layout/', 'scripts/bin/']
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
