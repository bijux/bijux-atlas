#!/usr/bin/env python3
# Purpose: enforce allowed generated directories policy.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
ALLOWED = {
    "docs/_generated",
    "ops/_generated",
    "ops/_generated_committed",
}

errors: list[str] = []
for p in ROOT.rglob("_generated"):
    if not p.is_dir():
        continue
    rel = p.relative_to(ROOT).as_posix()
    if rel in ALLOWED:
        continue
    if rel.startswith("artifacts/"):
        continue
    errors.append(rel)

if errors:
    print("generated dirs policy check failed", file=sys.stderr)
    for rel in sorted(errors):
        print(f"- disallowed generated dir: {rel}", file=sys.stderr)
    raise SystemExit(1)

print("generated dirs policy check passed")
