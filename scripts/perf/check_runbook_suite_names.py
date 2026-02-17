#!/usr/bin/env python3
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[2]
manifest = json.loads((root / "ops/load/suites/suites.json").read_text())
runbook = (root / "docs/operations/runbooks/load-failure-triage.md").read_text()
missing = []
for suite in manifest.get("suites", []):
    name = suite.get("name")
    if name and name not in runbook:
        missing.append(name)

if missing:
    print("runbook missing suite names:", file=sys.stderr)
    for name in missing:
        print(f"- {name}", file=sys.stderr)
    raise SystemExit(1)

print("runbook suite-name coverage passed")
