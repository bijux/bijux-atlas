#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

raw_patterns = [
    re.compile(r"\./ops/[\w./-]+\.sh"),
    re.compile(r"\bops/[\w./-]+run_all\.sh\b"),
    re.compile(r"\bops/[\w./-]+scripts/[\w./-]+\.sh\b"),
]

errors: list[str] = []
for md in sorted(DOCS.rglob("*.md")):
    text = md.read_text(encoding="utf-8", errors="ignore")
    for i, line in enumerate(text.splitlines(), start=1):
      for pat in raw_patterns:
        if pat.search(line):
          errors.append(f"{md.relative_to(ROOT)}:{i}: raw ops script reference found")

if errors:
    print("docs make-only ops entrypoint check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("docs make-only ops entrypoint check passed")
