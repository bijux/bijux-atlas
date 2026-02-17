#!/usr/bin/env python3
# Purpose: generate markdown inventory of public make targets from `make help`.
# Inputs: output of `make help`.
# Outputs: docs/development/make-targets.md and docs/development/make-targets-inventory.md.
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT_MAIN = ROOT / "docs" / "development" / "make-targets.md"
OUT_COMPAT = ROOT / "docs" / "development" / "make-targets-inventory.md"

help_out = subprocess.check_output(["make", "help"], cwd=ROOT, text=True)

sections: dict[str, list[str]] = {}
current: str | None = None
for line in help_out.splitlines():
    if line.endswith(":") and not line.startswith("  "):
        current = line[:-1]
        sections[current] = []
        continue
    if current and line.startswith("  "):
        sections[current].extend(line.strip().split())

lines: list[str] = [
    "# Make Targets Inventory",
    "",
    "- Owner: `docs-governance`",
    "",
    "Generated from `make help`. Do not edit manually.",
    "",
]

for section, targets in sections.items():
    lines.append(f"## {section.title()}")
    lines.append("")
    for target in targets:
        lines.append(f"- `{target}`")
    lines.append("")

rendered = "\n".join(lines)
OUT_MAIN.write_text(rendered, encoding="utf-8")
OUT_COMPAT.write_text(rendered, encoding="utf-8")
print(OUT_MAIN)
