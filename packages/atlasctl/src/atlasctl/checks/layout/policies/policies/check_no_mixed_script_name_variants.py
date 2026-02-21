#!/usr/bin/env python3
# Purpose: forbid underscore+dash duplicate naming variants for executable scripts.
from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
SCOPES = [ROOT / "ops", ROOT / "scripts"]

by_key: dict[tuple[str, str, str], set[str]] = defaultdict(set)
for base in SCOPES:
    if not base.exists():
        continue
    for p in base.rglob("*.sh"):
        rel = p.relative_to(ROOT).as_posix()
        parent = p.parent.relative_to(ROOT).as_posix()
        normalized = p.stem.lower().replace("_", "-")
        by_key[(parent, normalized, p.suffix)].add(rel)

errors: list[str] = []
for (_parent, _normalized, _suffix), paths in sorted(by_key.items()):
    names = {Path(x).name for x in paths}
    dash = {n for n in names if "-" in n}
    underscore = {n for n in names if "_" in n}
    if dash and underscore:
        errors.append("mixed naming variants found: " + ", ".join(sorted(paths)))

if errors:
    print("mixed script naming variants check failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("mixed script naming variants check passed")
