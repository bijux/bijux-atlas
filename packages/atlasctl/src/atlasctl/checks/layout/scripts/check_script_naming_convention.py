#!/usr/bin/env python3
# Purpose: enforce shell kebab-case and python snake_case naming conventions.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
SHELL_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.sh$")
PY_RE = re.compile(r"^[a-z0-9]+(?:_[a-z0-9]+)*\.py$")

EXEMPT = {
    }

SHELL_SCOPE = [ROOT / "ops/run", ROOT / "scripts/areas/public/contracts"]
PY_SCOPE = [ROOT / "scripts/areas/public/contracts", ROOT / "ops/report"]

errors: list[str] = []
for base in SHELL_SCOPE:
    for p in base.rglob("*.sh"):
        rel = p.relative_to(ROOT).as_posix()
        if rel in EXEMPT:
            continue
        if not SHELL_RE.match(p.name):
            errors.append(f"{rel}: shell scripts in public scope must be kebab-case")

for base in PY_SCOPE:
    for p in base.rglob("*.py"):
        rel = p.relative_to(ROOT).as_posix()
        if rel in EXEMPT:
            continue
        if not PY_RE.match(p.name):
            errors.append(f"{rel}: python scripts in public scope must be snake_case")

if errors:
    print("script naming convention check failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("script naming convention check passed")
