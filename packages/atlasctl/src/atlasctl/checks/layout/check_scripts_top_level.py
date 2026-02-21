#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
SCRIPTS = ROOT / "scripts"
ALLOWED = {"README.md", "INDEX.md", "DEPRECATED.md", "bin", "areas", "lib"}


def main() -> int:
    entries = {p.name for p in SCRIPTS.iterdir()}
    extra = sorted(entries - ALLOWED)
    missing = sorted(ALLOWED - entries)
    if extra or missing:
        print("scripts top-level layout failed:", file=sys.stderr)
        for name in missing:
            print(f"- missing: scripts/{name}", file=sys.stderr)
        for name in extra:
            print(f"- unexpected: scripts/{name}", file=sys.stderr)
        return 1

    print("scripts top-level layout passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
