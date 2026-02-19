#!/usr/bin/env python3
# Purpose: enforce no empty directories policy in ops tree.
# Inputs: ops/** directories.
# Outputs: non-zero for empty dirs without INDEX.md.
from __future__ import annotations
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
SKIP_PREFIXES = (
    "ops/_artifacts/",
    "ops/_generated/",
)
errors: list[str] = []
for d in sorted((ROOT / "ops").rglob("*")):
    if not d.is_dir():
        continue
    rel = d.relative_to(ROOT).as_posix()
    if any(rel.startswith(prefix) for prefix in SKIP_PREFIXES):
        continue
    entries = [p for p in d.iterdir()]
    if entries:
        continue
    idx = d / "INDEX.md"
    if not idx.exists():
        errors.append(f"empty directory missing INDEX.md: {d.relative_to(ROOT)}")

if errors:
    print("no-empty-dirs check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("no-empty-dirs check passed")
