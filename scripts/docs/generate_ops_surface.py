#!/usr/bin/env python3
# Purpose: generate ops surface documentation from ops manifests.
# Inputs: ops/_meta/surface.json and ops/e2e/suites/suites.json.
# Outputs: docs/_generated/ops-surface.md.
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
surface = json.loads((ROOT / "ops/_meta/surface.json").read_text(encoding="utf-8"))
suites = json.loads((ROOT / "ops/e2e/suites/suites.json").read_text(encoding="utf-8"))
out = ROOT / "docs/_generated/ops-surface.md"

lines = ["# Ops Surface", "", "Generated from ops manifests.", "", "## Stable Entrypoints", ""]
for t in surface.get("entrypoints", []):
    lines.append(f"- `make {t}`")

lines.extend(["", "## E2E Suites", ""])
for suite in suites.get("suites", []):
    caps = ",".join(suite.get("required_capabilities", []))
    lines.append(f"- `{suite['id']}`: capabilities=`{caps}`")
    for scenario in suite.get("scenarios", []):
        budget = scenario.get("budget", {})
        lines.append(
            f"- scenario `{scenario['id']}`: runner=`{scenario['runner']}`, "
            f"budget(time_s={budget.get('expected_time_seconds')}, qps={budget.get('expected_qps')})"
        )

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(out)
