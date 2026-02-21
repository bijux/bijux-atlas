#!/usr/bin/env python3
# Purpose: enforce script bucket taxonomy and required metadata headers.
# Inputs: scripts/**/*.sh and scripts/**/*.py.
# Outputs: non-zero if script file sits outside approved bucket paths.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
SCRIPTS = ROOT / "scripts"

ALLOWED_PREFIXES = (
    "scripts/areas/public/",
    "packages/atlasctl/src/atlasctl/checks/layout/",
)
LEGACY_ALLOWED = (
    "packages/atlasctl/src/atlasctl/checks/layout/shell/check_no_root_dumping.sh",
)

violations: list[str] = []
for path in sorted(SCRIPTS.rglob("*")):
    if not path.is_file() or path.suffix not in {".sh", ".py"}:
        continue
    rel = path.relative_to(ROOT).as_posix()
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
