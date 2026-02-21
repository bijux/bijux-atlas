#!/usr/bin/env python3
# Purpose: detect duplicate script functionality by normalized script intent names.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
OPS = ROOT / "ops"

seen: dict[str, str] = {}
errors: list[str] = []

for p in sorted(OPS.rglob("*.sh")):
    rel = p.relative_to(ROOT).as_posix()
    stem = p.stem.lower().replace("_", "-")
    stem = re.sub(r"-+", "-", stem)
    # Use parent area + normalized stem as intent key.
    area = p.parent.relative_to(OPS).as_posix()
    key = f"{area}::{stem}"
    prior = seen.get(key)
    if prior and prior != rel:
        errors.append(f"duplicate intent in same area: {prior} <-> {rel}")
    else:
        seen[key] = rel

if errors:
    print("duplicate script intent check failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("duplicate script intent check passed")
