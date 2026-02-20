#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OPS = ROOT / "ops"

required = {
    "stack", "k8s", "obs", "load", "datasets", "e2e", "run",
    "_lib", "_meta", "_schemas", "_generated", "_generated_committed", "_artifacts",
    "CONTRACT.md", "INDEX.md", "README.md", "ERRORS.md",
}

allowed_extra = {"fixtures", "registry", "report", "_evidence"}
allowed_extra = allowed_extra | {"_lint"}
allowed = required | allowed_extra

errors: list[str] = []

present = {p.name for p in OPS.iterdir()}
for name in sorted(required - present):
    errors.append(f"missing required ops entry: ops/{name}")

for name in sorted(present - allowed):
    errors.append(f"unexpected ops entry: ops/{name}")

for p in OPS.iterdir():
    if p.is_symlink() and p.name not in {"README.md", "CONTRACT.md"}:
        errors.append(f"forbidden symlink under ops/: ops/{p.name}")

if errors:
    print("ops layout contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops layout contract passed")
