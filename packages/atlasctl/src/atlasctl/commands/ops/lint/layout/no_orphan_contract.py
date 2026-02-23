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
contracts = json.loads((ROOT / "ops/inventory/contracts.json").read_text())
registered = {entry["path"] for entry in contracts.get("contracts", [])}
ops_index = (ROOT / "ops/INDEX.md").read_text(encoding="utf-8")
errors: list[str] = []

for rel in sorted(registered):
    if not rel.endswith("CONTRACT.md"):
        continue
    if not (ROOT / rel).exists():
        errors.append(f"registered contract missing on disk: {rel}")
    if rel != "ops/CONTRACT.md" and rel not in ops_index:
        errors.append(f"ops/INDEX.md missing contract reference: {rel}")

for path in sorted((ROOT / "ops").glob("*/CONTRACT.md")):
    rel = path.relative_to(ROOT).as_posix()
    if rel not in registered:
        errors.append(f"orphan contract not listed in contracts.json: {rel}")

if errors:
    for e in sorted(set(errors)):
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("contract registry is complete")
