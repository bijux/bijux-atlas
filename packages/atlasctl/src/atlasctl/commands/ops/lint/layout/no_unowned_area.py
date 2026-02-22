#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
ownership = json.loads((ROOT / "ops/_meta/ownership.json").read_text())
owned = sorted(ownership.get("areas", {}).keys())
errors: list[str] = []

for path in sorted((ROOT / "ops").glob("*")):
    if not path.is_dir():
        continue
    rel = path.relative_to(ROOT).as_posix()
    if rel in {"ops/_artifacts", "ops/_generated", "ops/_lib", "ops/_meta", "ops/_schemas", "ops/_lint", "ops/registry"}:
        continue
    if rel not in owned:
        errors.append(f"unowned area: {rel} (missing in ops/_meta/ownership.json)")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("all ops areas are owned")
