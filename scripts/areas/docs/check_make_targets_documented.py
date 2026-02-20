#!/usr/bin/env python3
# Purpose: verify every public make target from `make help` is documented in canonical docs pages.
# Inputs: repository Makefile help output and docs/development/makefiles/surface.md + docs/development/make-targets.md.
# Outputs: non-zero exit on undocumented targets; zero on full coverage.
from __future__ import annotations
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
surface_doc = (ROOT / 'docs' / 'development' / 'makefiles' / 'surface.md').read_text(encoding='utf-8')
targets_doc = (ROOT / 'docs' / 'development' / 'make-targets.md').read_text(encoding='utf-8')
help_out = subprocess.run(['make', 'help'], cwd=ROOT, text=True, capture_output=True, check=False).stdout.strip().splitlines()

if len(help_out) < 3:
    print('make target docs check failed: unexpected `make help` output', file=sys.stderr)
    raise SystemExit(1)

missing = []
for line in help_out:
    # Target lines are indented under namespace headers and include descriptions,
    # e.g. "    docs/all      Docs lane". Keep only the first token.
    if not line.startswith('    '):
        continue
    token = line.strip().split()[0]
    # Skip namespace markers like "[docs]".
    if token.startswith('[') and token.endswith(']'):
        continue
    if f'`{token}`' not in surface_doc and f'`{token}`' not in targets_doc:
        missing.append(token)

if missing:
    print('make target docs check failed:', file=sys.stderr)
    for t in missing:
        print(f'- docs/development/make-targets.md and docs/development/makefiles/surface.md missing `{t}`', file=sys.stderr)
    raise SystemExit(1)

print('make target docs check passed')
