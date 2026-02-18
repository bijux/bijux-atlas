#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
errors: list[str] = []

# Tool versions must be SSOT in configs/ops/tool-versions.json.
for path in (ROOT / "ops").rglob("*versions*.json"):
    rel = path.relative_to(ROOT).as_posix()
    if rel == "ops/stack/versions.json":
        continue
    errors.append(f"shadow versions config detected: {rel}")

# Observability pack config must be SSOT in configs/ops/observability-pack.json.
obs_cfg = [p.relative_to(ROOT).as_posix() for p in (ROOT / "ops").rglob("*observability*pack*.json")]
for rel in obs_cfg:
    errors.append(f"shadow observability pack config detected under ops: {rel}")

if errors:
    for e in sorted(set(errors)):
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("no shadow ops config sources detected")
