#!/usr/bin/env python3
# Purpose: generate markdown inventory of public make/ops commands from make help output.
# Inputs: make help output rendered via scripts/areas/layout/render_public_help.py.
# Outputs: docs/development/make-targets.md and docs/development/make-targets-inventory.md.
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT_MAIN = ROOT / "docs" / "development" / "make-targets.md"
OUT_COMPAT = ROOT / "docs" / "development" / "make-targets-inventory.md"
HELP_CMD = ["python3", "scripts/areas/layout/render_public_help.py"]


def parse_help_sections(text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.endswith(":") and not line.startswith("  "):
            current = line[:-1].strip()
            sections[current] = []
            continue
        if current and line.startswith("  "):
            target = line.strip().split()[0]
            sections[current].append(target)
    return sections


help_text = subprocess.check_output(HELP_CMD, cwd=ROOT, text=True)
sections = parse_help_sections(help_text)

lines: list[str] = [
    "# Make Targets Inventory",
    "",
    "- Owner: `docs-governance`",
    "",
    "Generated from `make help`. Do not edit manually.",
    "",
]

for section, section_targets in sections.items():
    lines.append(f"## {section}")
    lines.append("")
    for target in section_targets:
        lines.append(f"- `{target}`")
    lines.append("")

rendered = "\n".join(lines)
OUT_MAIN.write_text(rendered, encoding="utf-8")
OUT_COMPAT.write_text(rendered, encoding="utf-8")
print(OUT_MAIN)
