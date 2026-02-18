#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
errors: list[str] = []

for md in sorted((ROOT / "ops").rglob("README.md")):
    rel = md.relative_to(ROOT).as_posix()
    if rel == "ops/README.md":
        continue
    text = md.read_text(encoding="utf-8", errors="ignore")
    if "ops/README.md" not in text and "docs/operations/INDEX.md" not in text:
        errors.append(f"{rel}: missing canonical link to ops/README.md or docs/operations/INDEX.md")

if errors:
    print("ops README canonical-link check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops README canonical-link check passed")
