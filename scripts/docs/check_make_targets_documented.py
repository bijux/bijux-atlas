#!/usr/bin/env python3
# Purpose: verify every public make target from `make help` is documented in docs/development/makefiles/surface.md.
# Inputs: repository Makefile help output and docs/development/makefiles/surface.md.
# Outputs: non-zero exit on undocumented targets; zero on full coverage.
from __future__ import annotations
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
doc = (ROOT / 'docs' / 'development' / 'makefiles' / 'surface.md').read_text(encoding='utf-8')
help_out = subprocess.run(['make', 'help'], cwd=ROOT, text=True, capture_output=True, check=False).stdout.strip().splitlines()

if len(help_out) < 3:
    print('make target docs check failed: unexpected `make help` output', file=sys.stderr)
    raise SystemExit(1)

missing = []
for line in help_out:
    parts = line.split(':', 1)
    if len(parts) != 2:
        continue
    targets = [t for t in parts[1].strip().split() if t]
    for t in targets:
        if f'`{t}`' not in doc:
            missing.append(t)

if missing:
    print('make target docs check failed:', file=sys.stderr)
    for t in missing:
        print(f'- docs/development/makefiles/surface.md missing `{t}`', file=sys.stderr)
    raise SystemExit(1)

print('make target docs check passed')
