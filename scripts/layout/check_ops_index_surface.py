#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
index_text = (ROOT / "ops" / "INDEX.md").read_text(encoding="utf-8")
surface = json.loads((ROOT / "ops" / "_meta" / "surface.json").read_text(encoding="utf-8"))

missing: list[str] = []
for t in surface.get("entrypoints", []):
    if f"make {t}" not in index_text and f"`{t}`" not in index_text:
        missing.append(t)

if missing:
    print("ops index surface contract failed:", file=sys.stderr)
    for t in missing:
        print(f"- missing in ops/INDEX.md: {t}", file=sys.stderr)
    raise SystemExit(1)

print("ops index surface contract passed")
