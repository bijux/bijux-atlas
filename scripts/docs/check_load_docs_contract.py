#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import re
import json
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
suites_manifest = ROOT / "ops" / "load" / "suites" / "suites.json"
suite_ids: set[str] = set()
if suites_manifest.exists():
    data = json.loads(suites_manifest.read_text(encoding="utf-8"))
    suite_ids = {item["name"] for item in data.get("suites", []) if isinstance(item, dict) and "name" in item}

errors: list[str] = []
for path in sorted(LOAD_DOC_DIR.glob("*.md")):
    text = path.read_text(encoding="utf-8")
    refs = re.findall(r"`([a-zA-Z0-9_\-]+\.json)`", text)
    refs = [ref for ref in refs if ref != "suites.json"]
    suite_refs = re.findall(r"`([a-z0-9][a-z0-9\-]+)`", text)
    suite_refs = [ref for ref in suite_refs if ref in suite_ids]
    if not refs and not suite_refs:
        errors.append(f"{path}: no load suite or k6 scenario references found")
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
