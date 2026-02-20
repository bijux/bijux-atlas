#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OWNERS = json.loads((ROOT / "ops/_meta/ownership.json").read_text(encoding="utf-8")).get("areas", {})
EXCLUDE_DIRS = {"ops/_generated", "ops/_artifacts"}


def owner_for(path: Path) -> str | None:
    rel = path.relative_to(ROOT).as_posix()
    for area in sorted(OWNERS.keys(), key=len, reverse=True):
        if rel == area or rel.startswith(area + "/"):
            return OWNERS[area]
    return None


def main() -> int:
    errors: list[str] = []
    for p in sorted((ROOT / "ops").rglob("*")):
        if p.is_dir():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if any(rel == d or rel.startswith(d + "/") for d in EXCLUDE_DIRS):
            continue
        if owner_for(p) is None:
            errors.append(rel)

    if errors:
        print("unowned ops files detected:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("all ops files have ownership coverage")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
