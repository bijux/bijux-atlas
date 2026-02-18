#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ownership = json.loads((ROOT / "ops/_meta/ownership.json").read_text())
contracts = json.loads((ROOT / "ops/_meta/contracts.json").read_text())
registered = {entry["path"] for entry in contracts.get("contracts", [])}
errors: list[str] = []

for area in ownership.get("areas", {}):
    contract = f"{area}/CONTRACT.md"
    if not (ROOT / contract).exists():
        errors.append(f"missing contract file for area: {contract}")
    if contract not in registered:
        errors.append(f"contract not registered in ops/_meta/contracts.json: {contract}")

for path in sorted((ROOT / "ops").glob("*/CONTRACT.md")):
    rel = path.relative_to(ROOT).as_posix()
    if rel not in registered:
        errors.append(f"orphan contract not listed in contracts.json: {rel}")

if errors:
    for e in sorted(set(errors)):
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("contract registry is complete")
