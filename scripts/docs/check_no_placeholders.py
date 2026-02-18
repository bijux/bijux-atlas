#!/usr/bin/env python3
# Purpose: fail docs lint on placeholder words outside docs/_drafts.
# Inputs: docs/**/*.md content.
# Outputs: non-zero if TODO/TBD/placeholder markers are found.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
pat = re.compile(r"\b(TODO|TBD|placeholder|coming soon)\b", re.IGNORECASE)
violations: list[str] = []
for md in sorted(DOCS.rglob("*.md")):
    rel = md.relative_to(ROOT).as_posix()
    if rel.startswith("docs/_drafts/"):
        continue
    for i, line in enumerate(md.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        if pat.search(line):
            violations.append(f"{rel}:{i}: placeholder marker")

if violations:
    print("docs placeholder check failed:", file=sys.stderr)
    for v in violations[:200]:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("docs placeholder check passed")
