#!/usr/bin/env python3
# Purpose: generate ops surface documentation from ops manifests.
# Inputs: ops/_meta/surface.json and ops/e2e/scenarios/scenarios.json.
# Outputs: docs/_generated/ops-surface.md.
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
surface = json.loads((ROOT / "ops/_meta/surface.json").read_text(encoding="utf-8"))
scenarios = json.loads((ROOT / "ops/e2e/scenarios/scenarios.json").read_text(encoding="utf-8"))
out = ROOT / "docs/_generated/ops-surface.md"

lines = ["# Ops Surface", "", "Generated from ops manifests.", "", "## Stable Entrypoints", ""]
for t in surface.get("entrypoints", []):
    lines.append(f"- `make {t}`")

lines.extend(["", "## E2E Scenarios", ""])
for s in scenarios.get("scenarios", []):
    comp = s.get("compose", {})
    lines.append(f"- `{s['id']}`: `{s['entrypoint']}` (stack={comp.get('stack')}, obs={comp.get('obs')}, datasets={comp.get('datasets')}, load={comp.get('load')})")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(out)
