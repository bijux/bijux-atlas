#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import re
import sys
from pathlib import Path

DOCS = Path('docs')
errors = []

# terminology gate
term_bans = {
    r'\bgenome build\b': 'assembly',
    r'\bwhitelist\b': 'allowlist',
    r'\bblacklist\b': 'denylist',
}

# units gate (heuristic): require units when discussing coordinate/size/latency values
units_pat = re.compile(
    r'\b(coordinate|span|size|latency|timeout)\b[^\n]{0,40}\b(?<![pP.])(\d{2,})\b(?!\s*(bp|bytes|seconds|ms|s))(?!\.)',
    re.IGNORECASE,
)

# ssot references only for registries
ssot_ban = re.compile(r'docs/contracts/(ERROR_CODES|METRICS|TRACE_SPANS|ENDPOINTS|CONFIG_KEYS|CHART_VALUES)\.json')

for p in DOCS.rglob('*.md'):
    txt = p.read_text(encoding='utf-8')
    if p.name == 'terms-glossary.md':
        continue
    for pat,repl in term_bans.items():
        if re.search(pat, txt, flags=re.IGNORECASE):
            errors.append(f"{p}: terminology violation; use `{repl}`")
    if 'reference' in p.parts or 'product' in p.parts or 'operations' in p.parts:
        if units_pat.search(txt):
            errors.append(f"{p}: possible missing unit annotation (bp/bytes/seconds)")
    if 'contracts' not in p.parts and ssot_ban.search(txt):
        errors.append(f"{p}: reference docs/contracts/*.md instead of raw registry json")

if errors:
    print('terminology/units/ssot check failed:', file=sys.stderr)
    for e in errors:
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)

print('terminology/units/ssot check passed')
