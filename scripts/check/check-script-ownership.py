#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ownership = json.loads((ROOT / "scripts/_meta/ownership.json").read_text(encoding="utf-8"))["areas"]

errors: list[str] = []
for p in sorted((ROOT / "scripts").rglob("*")):
    if not p.is_file():
        continue
    rel = p.relative_to(ROOT).as_posix()
    if rel.startswith("scripts/__pycache__"):
        continue
    matched = any(rel == area or rel.startswith(area + "/") for area in ownership)
    if not matched:
        errors.append(rel)

if errors:
    print("script ownership coverage failed:", file=sys.stderr)
    for rel in errors:
        print(f"- {rel}", file=sys.stderr)
    raise SystemExit(1)

print("script ownership coverage passed")
