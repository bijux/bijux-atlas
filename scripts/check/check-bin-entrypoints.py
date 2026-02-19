#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
files = sorted(p for p in (ROOT / "scripts/bin").glob("*") if p.is_file())
if len(files) > 15:
    print(f"scripts/bin cap exceeded: {len(files)} > 15", file=sys.stderr)
    raise SystemExit(1)
print(f"scripts/bin cap ok: {len(files)}")
