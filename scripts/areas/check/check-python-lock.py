#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
locks = [
    ROOT / "scripts/areas/python/requirements.lock.txt",
    ROOT / "tools/bijux-atlas-scripts/requirements.lock.txt",
]
pat = re.compile(r"^[a-zA-Z0-9_.-]+==[a-zA-Z0-9_.-]+$")
all_errors: list[str] = []
for lock in locks:
    text = lock.read_text(encoding="utf-8")
    lines = [ln.strip() for ln in text.splitlines() if ln.strip() and not ln.strip().startswith("#")]
    errors = [ln for ln in lines if not pat.match(ln)]
    for err in errors:
        all_errors.append(f"{lock.relative_to(ROOT)}: {err}")

if all_errors:
    print("invalid scripts requirements lock entries:", file=sys.stderr)
    for ln in all_errors:
        print(f"- {ln}", file=sys.stderr)
    raise SystemExit(1)
print("scripts python lock format passed")
