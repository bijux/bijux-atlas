#!/usr/bin/env python3
# Purpose: ensure generated make targets docs are committed and up to date.
# Inputs: scripts/areas/docs/generate_make_targets_inventory.py output and git diff.
# Outputs: non-zero on drift.
from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
paths = ["docs/development/make-targets.md", "docs/development/make-targets-inventory.md"]


def digest(path: Path) -> str:
    if not path.exists():
        return ""
    return hashlib.sha256(path.read_bytes()).hexdigest()


before = {p: digest(ROOT / p) for p in paths}
subprocess.run(["python3", "scripts/areas/docs/generate_make_targets_inventory.py"], cwd=ROOT, check=True)
after = {p: digest(ROOT / p) for p in paths}

if before != after:
    print("make-target docs drift detected; regenerate and commit:", file=sys.stderr)
    print("python3 scripts/areas/docs/generate_make_targets_inventory.py", file=sys.stderr)
    raise SystemExit(1)

print("make-target docs drift check passed")
