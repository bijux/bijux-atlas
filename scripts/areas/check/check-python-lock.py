#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

lock = Path(__file__).resolve().parents[3] / "scripts/areas/python/requirements.lock.txt"
text = lock.read_text(encoding="utf-8")
lines = [ln.strip() for ln in text.splitlines() if ln.strip() and not ln.strip().startswith("#")]
pat = re.compile(r"^[a-zA-Z0-9_.-]+==[a-zA-Z0-9_.-]+$")
errors = [ln for ln in lines if not pat.match(ln)]
if errors:
    print("invalid scripts requirements lock entries:", file=sys.stderr)
    for ln in errors:
        print(f"- {ln}", file=sys.stderr)
    raise SystemExit(1)
print("scripts python lock format passed")
