#!/usr/bin/env python3
# Purpose: enforce full-stack docs page size and target drift policy.
# Inputs: docs/operations/full-stack-local.md and makefiles.
# Outputs: non-zero exit on policy violation.
from __future__ import annotations
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
page = ROOT / 'docs' / 'operations' / 'full-stack-local.md'
text = page.read_text()
lines = [l for l in text.splitlines() if l.strip()]
if len(lines) > 80:
    raise SystemExit('full-stack page exceeds one-page policy (>80 non-empty lines)')
required = 'make ops-up ops-deploy ops-warm ops-smoke'
if required not in text:
    raise SystemExit('full-stack page missing canonical command sequence')
mk = (ROOT / 'makefiles' / 'ops.mk').read_text()
for target in ['ops-up', 'ops-deploy', 'ops-warm', 'ops-smoke']:
    if not re.search(rf'^{target}:', mk, flags=re.M):
        raise SystemExit(f'missing target in ops.mk: {target}')
print('full stack page check passed')
