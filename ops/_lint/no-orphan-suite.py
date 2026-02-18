#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
manifest = json.loads((ROOT / "ops/load/suites/suites.json").read_text())
scenarios_dir = ROOT / manifest["scenarios_dir"]
errors: list[str] = []

scenario_files = {p.name for p in scenarios_dir.glob("*.json")}
manifest_scenarios = {
    s.get("scenario") for s in manifest.get("suites", []) if s.get("kind") == "k6"
}
missing = sorted(s for s in manifest_scenarios if s and s not in scenario_files)
for s in missing:
    errors.append(f"missing scenario file referenced by suites.json: {s}")

referenced_js: set[str] = set()
for scenario in scenarios_dir.glob("*.json"):
    payload = json.loads(scenario.read_text())
    suite = payload.get("suite")
    if isinstance(suite, str) and suite.endswith(".js"):
        referenced_js.add(suite)

for js in sorted((ROOT / "ops/load/k6/suites").glob("*.js")):
    if js.name not in referenced_js:
        errors.append(f"orphan k6 suite not referenced by any scenario: {js.relative_to(ROOT).as_posix()}")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("all load suites are referenced")
