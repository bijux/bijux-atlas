#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOAD_DOC_DIR = ROOT / "docs" / "operations" / "load"
scenario_dir = ROOT / "ops" / "load" / "scenarios"
if not scenario_dir.exists():
    scenario_dir = ROOT / "ops" / "e2e" / "k6" / "scenarios"
if not scenario_dir.exists():
    scenario_dir = ROOT / "e2e" / "k6" / "scenarios"
scenarios = {p.name for p in scenario_dir.glob("*.json")}

errors: list[str] = []
for path in sorted(LOAD_DOC_DIR.glob("*.md")):
    text = path.read_text(encoding="utf-8")
    refs = re.findall(r"`([a-zA-Z0-9_\-]+\.json)`", text)
    refs = [ref for ref in refs if ref != "suites.json"]
    if not refs:
        errors.append(f"{path}: no k6 scenario references found")
        continue
    for ref in refs:
        if ref not in scenarios:
            errors.append(f"{path}: unknown k6 scenario `{ref}`")

if errors:
    print("load docs contract check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    sys.exit(1)

print("load docs contract check passed")
