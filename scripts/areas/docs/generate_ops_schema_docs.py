#!/usr/bin/env python3
# Purpose: generate ops schema inventory docs from ops/_schemas SSOT.
# Inputs: ops/_schemas/**/*.json.
# Outputs: docs/_generated/ops-schemas.md.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMAS = sorted((ROOT / "ops" / "_schemas").rglob("*.json"))
OUT = ROOT / "docs" / "_generated" / "ops-schemas.md"

lines = [
    "# Ops Schemas",
    "",
    "Generated from `ops/_schemas`. Do not edit manually.",
    "",
]
for p in SCHEMAS:
    rel = p.relative_to(ROOT).as_posix()
    try:
        payload = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        payload = {}
    req = payload.get("required", []) if isinstance(payload, dict) else []
    lines.append(f"## `{rel}`")
    lines.append("")
    if req:
        lines.append("Required keys:")
        for key in req:
            lines.append(f"- `{key}`")
    else:
        lines.append("Required keys: none")
    lines.append("")

OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(OUT)
