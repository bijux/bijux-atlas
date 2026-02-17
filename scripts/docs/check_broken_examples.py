#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path('.').resolve()
DOCS = ROOT / 'docs'
codeblock = re.compile(r'```(?:bash|sh)\n(.*?)```', re.DOTALL)
cmdline = re.compile(r'^\$\s+(.+)$', re.MULTILINE)

make_db = subprocess.run(['make', '-qp'], cwd=ROOT, text=True, capture_output=True, check=False).stdout
make_targets = set(re.findall(r'^([A-Za-z0-9_.%/+\-]+):', make_db, flags=re.MULTILINE))

errors = []
for md in DOCS.rglob('*.md'):
    text = md.read_text(encoding='utf-8')
    for block in codeblock.findall(text):
        for cmd in cmdline.findall(block):
            tok = cmd.strip().split()[0]
            if tok == 'make':
                parts = cmd.strip().split()
                if len(parts) < 2 or parts[1] not in make_targets:
                    errors.append(f"{md}: unknown make target in example `{cmd}`")
                continue
            if tok.startswith('./'):
                p = (ROOT / tok).resolve()
                if not p.exists() or not p.is_file() or not (p.stat().st_mode & 0o111):
                    errors.append(f"{md}: non-executable script path `{tok}`")
                continue
            if tok in {'curl','kubectl','k6','cargo','rg','python3','helm'}:
                continue
            errors.append(f"{md}: command not backed by script path or allowed tool `{cmd}`")

if errors:
    print('broken examples check failed:', file=sys.stderr)
    for e in errors:
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)

print('broken examples check passed')
