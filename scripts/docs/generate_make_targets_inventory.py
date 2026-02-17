#!/usr/bin/env python3
# Purpose: generate markdown inventory of public make targets from makefile registry.
# Inputs: makefiles/registry.mk.
# Outputs: docs/development/make-targets.md and docs/development/make-targets-inventory.md.
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT_MAIN = ROOT / "docs" / "development" / "make-targets.md"
OUT_COMPAT = ROOT / "docs" / "development" / "make-targets-inventory.md"
REGISTRY = ROOT / "makefiles" / "registry.mk"

sections: dict[str, list[str]] = {}
desc_re = re.compile(r"^REGISTRY_([A-Z_]+)_DESC := (.+)$")
targets_re = re.compile(r"^REGISTRY_([A-Z_]+)_TARGETS := (.*)$")
descs: dict[str, str] = {}
targets: dict[str, list[str]] = {}

for line in REGISTRY.read_text(encoding="utf-8").splitlines():
    match = desc_re.match(line.strip())
    if match:
        descs[match.group(1)] = match.group(2).strip()
        continue
    match = targets_re.match(line.strip())
    if match:
        targets[match.group(1)] = [token for token in match.group(2).split() if token]

for key, desc in descs.items():
    sections[desc] = targets.get(key, [])

lines: list[str] = [
    "# Make Targets Inventory",
    "",
    "- Owner: `docs-governance`",
    "",
    "Generated from `make help`. Do not edit manually.",
    "",
]

for section, section_targets in sections.items():
    lines.append(f"## {section.title()}")
    lines.append("")
    for target in section_targets:
        lines.append(f"- `{target}`")
    lines.append("")

rendered = "\n".join(lines)
OUT_MAIN.write_text(rendered, encoding="utf-8")
OUT_COMPAT.write_text(rendered, encoding="utf-8")
print(OUT_MAIN)
