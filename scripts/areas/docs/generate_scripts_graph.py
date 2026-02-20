#!/usr/bin/env python3
# Purpose: generate make-target to script call graph documentation.
# Inputs: Makefile and makefiles/*.mk.
# Outputs: docs/development/scripts-graph.md.
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
mk_files = [ROOT / "Makefile", *sorted((ROOT / "makefiles").glob("*.mk"))]
out = ROOT / "docs" / "development" / "scripts-graph.md"

target_re = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)")
script_re = re.compile(r"(?:\./|python3\s+|python\s+)(scripts/(?:public|internal)/[^\s\"']+)")

rows: list[tuple[str, str]] = []
for mk in mk_files:
    current = ""
    for line in mk.read_text(encoding="utf-8").splitlines():
        if line.startswith("\t"):
            for m in script_re.finditer(line):
                rows.append((current, m.group(1).rstrip(";")))
            continue
        m = target_re.match(line)
        if m and not line.startswith("."):
            current = m.group(1)

rows = sorted(set((t, s) for t, s in rows if t and s))

lines = [
    "# Scripts Graph",
    "",
    "Generated file. Do not edit by hand.",
    "",
    "| Make Target | Script |",
    "|---|---|",
]
for target, script in rows:
    lines.append(f"| `{target}` | `{script}` |")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"wrote {out.relative_to(ROOT)}")
