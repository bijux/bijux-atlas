#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OBS_SCRIPTS = ROOT / "ops/obs/scripts"


def canonical(name: str) -> str:
    stem = name[:-3] if name.endswith(".sh") else name
    return stem.replace("_", "-")


def main() -> int:
    files = sorted(p.name for p in OBS_SCRIPTS.glob("*.sh"))
    seen: dict[str, str] = {}
    errors: list[str] = []
    for name in files:
        key = canonical(name)
        if key in seen and seen[key] != name:
            errors.append(f"duplicate obs script intent: {seen[key]} vs {name} (canonical `{key}`)")
        else:
            seen[key] = name

    if errors:
        print("observability script naming collision check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("observability script naming collision check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
