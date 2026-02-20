#!/usr/bin/env python3
# Purpose: enforce committed generated-files policy for ops generated outputs.
# Inputs: ops/_generated_committed and docs/_generated ops artifacts.
# Outputs: non-zero if unexpected/missing generated files are detected.
from __future__ import annotations
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
expected = {
    "ops/_generated_committed/.gitkeep",
    "ops/_generated_committed/examples/report.example.json",
    "ops/_generated_committed/examples/report.unified.example.json",
    "docs/_generated/ops-surface.md",
    "docs/_generated/ops-contracts.md",
    "docs/_generated/ops-schemas.md",
    "docs/_generated/ops-badge.md",
    "ops/_generated_committed/scorecard.json",
}
actual = set()
for rel in expected:
    if (ROOT / rel).exists():
        actual.add(rel)
missing = sorted(expected - actual)
unknown = sorted(
    p.relative_to(ROOT).as_posix()
    for p in (ROOT / "ops/_generated_committed").rglob("*")
    if p.is_file()
    and p.relative_to(ROOT).as_posix() != "ops/_generated_committed/report.unified.json"
    and p.relative_to(ROOT).as_posix() not in expected
)
unknown.extend(
    p.relative_to(ROOT).as_posix()
    for p in (ROOT / "docs/_generated").glob("ops-*.md")
    if p.is_file() and p.relative_to(ROOT).as_posix() not in expected
)
if missing or unknown:
    print("generated policy check failed:", file=sys.stderr)
    for m in missing:
        print(f"- missing expected generated file: {m}", file=sys.stderr)
    for u in sorted(unknown):
        print(f"- unexpected generated file: {u}", file=sys.stderr)
    raise SystemExit(1)
print("generated policy check passed")
