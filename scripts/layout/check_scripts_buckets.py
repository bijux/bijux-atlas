#!/usr/bin/env python3
# Purpose: enforce script bucket taxonomy and required metadata headers.
# Inputs: scripts/**/*.sh and scripts/**/*.py.
# Outputs: non-zero if script file sits outside approved bucket paths.
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"

ALLOWED_PREFIXES = (
    "scripts/public/",
    "scripts/internal/",
    "scripts/dev/",
    "scripts/docs/",
    "scripts/layout/",
    "scripts/contracts/",
    "scripts/release/",
    "scripts/fixtures/",
    "scripts/bootstrap/",
    "scripts/bin/",
    "scripts/ops/",
    "scripts/tools/",
    "scripts/tests/",
    "scripts/demo/",
)
LEGACY_ALLOWED = (
    "scripts/generate_scripts_readme.py",
)

violations: list[str] = []
for path in sorted(SCRIPTS.rglob("*")):
    if not path.is_file() or path.suffix not in {".sh", ".py"}:
        continue
    rel = path.relative_to(ROOT).as_posix()
    if rel.startswith("scripts/_internal/"):
        continue
    if rel in LEGACY_ALLOWED:
        continue
    if any(rel.startswith(prefix) for prefix in ALLOWED_PREFIXES):
        continue
    violations.append(rel)

if violations:
    print("scripts bucket check failed:", file=sys.stderr)
    for rel in violations:
        print(f"- {rel}: move under scripts/public|internal|dev or approved domain buckets", file=sys.stderr)
    raise SystemExit(1)

print("scripts bucket check passed")
