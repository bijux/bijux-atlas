#!/usr/bin/env python3
# Purpose: ensure `make help` output matches registry-generated targets inventory.
# Inputs: make help output and docs/development/make-targets.md.
# Outputs: non-zero on mismatch between help and generated docs.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOC = ROOT / "docs" / "development" / "make-targets.md"


def parse_help(text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.endswith(":") and not line.startswith("  "):
            current = line[:-1]
            sections[current] = []
            continue
        if current and line.startswith("  "):
            # Keep only the first token (target name or grouping marker),
            # matching scripts/areas/docs/generate_make_targets_inventory.py.
            sections[current].append(line.strip().split()[0])
    return sections


def parse_doc(text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.startswith("## "):
            current = line[3:].strip().lower()
            sections[current] = []
            continue
        if current:
            match = re.match(r"^- `([^`]+)`$", line.strip())
            if match:
                sections[current].append(match.group(1))
    return sections

help_sections = parse_help(subprocess.check_output(["make", "help"], cwd=ROOT, text=True))
doc_sections = parse_doc(DOC.read_text(encoding="utf-8"))

normalized_help = {k.lower(): v for k, v in help_sections.items()}
if normalized_help != doc_sections:
    print("make help drift detected vs docs/development/make-targets.md", file=sys.stderr)
    raise SystemExit(1)

print("make help drift check passed")
