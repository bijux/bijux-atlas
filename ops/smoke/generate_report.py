#!/usr/bin/env python3
# Purpose: generate smoke report markdown from run artifacts.
# Inputs: artifacts/ops/<run-id>/smoke/{requests.log,responses.jsonl}.
# Outputs: artifacts/ops/<run-id>/smoke/report.md.
from __future__ import annotations

import json
import os
from pathlib import Path

root = Path(__file__).resolve().parents[2]
run_id = os.environ.get("OPS_RUN_ID") or os.environ.get("ATLAS_RUN_ID") or "local"
run_dir = Path(os.environ.get("OPS_RUN_DIR", root / "artifacts" / "ops" / run_id))
smoke = run_dir / "smoke"
smoke.mkdir(parents=True, exist_ok=True)
responses = smoke / "responses.jsonl"
report = smoke / "report.md"
rows: list[dict[str, object]] = []
if responses.exists():
    for line in responses.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        rows.append(json.loads(line))

lines = ["# Ops Smoke Report", "", f"- Run: `{run_id}`", "", "| Path | Status |", "|---|---|"]
for row in rows:
    lines.append(f"| `{row.get('path','')}` | `{row.get('status','')}` |")
if not rows:
    lines.append("| _none_ | _none_ |")

report.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(report)
