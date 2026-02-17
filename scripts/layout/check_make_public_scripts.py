#!/usr/bin/env python3
# Purpose: ensure make recipes call only scripts declared as public entrypoints.
# Inputs: Makefile + makefiles/*.mk and scripts/ENTRYPOINTS.md patterns.
# Outputs: non-zero exit when make calls non-public scripts.
from __future__ import annotations
import fnmatch
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
entry = (ROOT / "scripts/ENTRYPOINTS.md").read_text().splitlines()
patterns: list[str] = []
in_public = False
for line in entry:
    if line.strip() == "## Public":
        in_public = True
        continue
    if line.startswith("## ") and line.strip() != "## Public":
        in_public = False
    if in_public and line.strip().startswith("- `"):
        patterns.append(line.strip()[3:-1].split(" ")[0])

mk_files = [ROOT / "Makefile"] + sorted((ROOT / "makefiles").glob("*.mk"))
text = "\n".join(p.read_text() for p in mk_files)
called = set(re.findall(r"\./(scripts/[^\s\"\\]+)", text))
called.update(re.findall(r"(?:python3|python)\s+(scripts/[^\s\"\\]+)", text))
called = sorted(path.rstrip(";") for path in called)

violations: list[str] = []
for path in called:
    if not any(fnmatch.fnmatch(path, pat) for pat in patterns):
        violations.append(path)

if violations:
    print("make references non-public scripts:")
    for v in violations:
        print(f"- {v}")
    raise SystemExit(1)
print("make public script gate passed")
