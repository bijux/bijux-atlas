#!/usr/bin/env python3
# Purpose: ensure generated make targets docs are committed and up to date.
# Inputs: scripts/docs/generate_make_targets_inventory.py output and git diff.
# Outputs: non-zero on drift.
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

subprocess.run(["python3", "scripts/docs/generate_make_targets_inventory.py"], cwd=ROOT, check=True)
paths = ["docs/development/make-targets.md", "docs/development/make-targets-inventory.md"]
proc = subprocess.run(["git", "diff", "--quiet", "--", *paths], cwd=ROOT)
if proc.returncode != 0:
    print("make-target docs drift detected; regenerate and commit:", file=sys.stderr)
    print("python3 scripts/docs/generate_make_targets_inventory.py", file=sys.stderr)
    raise SystemExit(1)

print("make-target docs drift check passed")
