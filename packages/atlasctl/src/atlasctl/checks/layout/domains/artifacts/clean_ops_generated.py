#!/usr/bin/env python3
# Purpose: clean ops generated outputs while preserving committed generated artifacts policy.
# Inputs: ops/_generated_committed and docs/_generated.
# Outputs: removes stale generated files outside committed allowlist.
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
allow = {
    "ops/_generated_committed/.gitkeep",
    "docs/_generated/ops-surface.md",
    "docs/_generated/ops-contracts.md",
    "docs/_generated/ops-schemas.md",
}
removed = 0
for base in (ROOT / "ops/_generated_committed", ROOT / "docs/_generated"):
    for p in sorted(base.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel in allow:
            continue
        if base == ROOT / "docs/_generated" and not rel.startswith("docs/_generated/ops-"):
            continue
        p.unlink()
        removed += 1
print(f"removed {removed} generated files")
