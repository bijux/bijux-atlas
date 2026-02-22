#!/usr/bin/env python3
# Purpose: enforce commands/dev first-level submodule budget.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
DEV_ROOT = ROOT / "packages" / "atlasctl" / "src" / "atlasctl" / "commands" / "dev"
MAX_SUBMODULES = 10


def main() -> int:
    modules = sorted(
        path.name
        for path in DEV_ROOT.iterdir()
        if path.is_dir() and not path.name.startswith("__")
    )
    if len(modules) > MAX_SUBMODULES:
        print("dev submodule budget check failed", file=sys.stderr)
        print(f"- commands/dev has {len(modules)} submodules (max {MAX_SUBMODULES})", file=sys.stderr)
        print(f"- submodules: {', '.join(modules)}", file=sys.stderr)
        return 1

    print("dev submodule budget check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
