#!/usr/bin/env python3
# Purpose: generate ops contracts documentation from schema inventory.
# Inputs: ops/_schemas/**/*.json and ops/_meta/contracts.json.
# Outputs: docs/_generated/ops-contracts.md.
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
out = ROOT / "docs/_generated/ops-contracts.md"
contracts = json.loads((ROOT / "ops/_meta/contracts.json").read_text(encoding="utf-8"))
schemas = sorted((ROOT / "ops/_schemas").rglob("*.json"))
lines = ["# Ops Contracts", "", "Generated from ops contracts and schemas.", "", "## Contract Files", ""]
for c in contracts.get("contracts", []):
    lines.append(f"- `{c['path']}` (version `{c['version']}`)")
lines.extend(["", "## Schemas", ""])
for s in schemas:
    lines.append(f"- `{s.relative_to(ROOT).as_posix()}`")
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(out)
