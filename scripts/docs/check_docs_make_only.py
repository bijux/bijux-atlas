#!/usr/bin/env python3
# Purpose: enforce docs reference make targets, not direct script paths, for operator workflows.
# Inputs: docs/**/*.md
# Outputs: non-zero when direct script calls are documented.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = sorted((ROOT / "docs" / "operations").rglob("*.md"))

patterns = [
    re.compile(r"(^|\s)(\./)?scripts/[\w./-]+"),
    re.compile(r"(^|\s)(\./)?ops/.+/scripts/[\w./-]+"),
]
exceptions = set()

violations: list[str] = []
for doc in DOCS:
    rel = str(doc.relative_to(ROOT))
    if rel in exceptions:
        continue
    for idx, line in enumerate(doc.read_text(encoding="utf-8").splitlines(), start=1):
        if line.strip().startswith("#"):
            continue
        if "`" not in line and "scripts/" not in line:
            continue
        for pat in patterns:
            if pat.search(line):
                violations.append(f"{rel}:{idx}: direct script path in docs; reference `make <target>` instead")
                break

if violations:
    print("docs make-only check failed:", file=sys.stderr)
    for v in violations[:200]:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("docs make-only check passed")
