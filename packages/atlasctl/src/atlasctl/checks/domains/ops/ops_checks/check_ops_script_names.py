#!/usr/bin/env python3
# Purpose: enforce durable noun+qualifier naming for ops scripts/manifests.
# Inputs: ops/**/*.sh and ops/**/*.json manifests.
# Outputs: non-zero when forbidden temporal/task naming appears.
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
forbidden = re.compile(r"\b(phase|task|stage|tmp|temp|final|draft|new|old|vnext)\b", re.IGNORECASE)
errors: list[str] = []

for p in sorted((ROOT / "ops").rglob("*.sh")):
    name = p.stem
    if forbidden.search(name):
        errors.append(f"forbidden temporal/task token in script name: {p.relative_to(ROOT)}")

for p in sorted((ROOT / "ops").rglob("*.json")):
    name = p.stem
    if forbidden.search(name):
        errors.append(f"forbidden temporal/task token in manifest name: {p.relative_to(ROOT)}")

if errors:
    print("ops naming durability check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("ops naming durability check passed")
