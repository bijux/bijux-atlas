#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CLI_MAIN = ROOT / "packages/atlasctl/src/atlasctl/cli/main.py"
MAX_LOC = 300


def main() -> int:
    lines = CLI_MAIN.read_text(encoding="utf-8").splitlines()
    loc = len(lines)
    if loc > MAX_LOC:
        print(f"cli main LOC budget exceeded: {loc} > {MAX_LOC}")
        return 1
    print("cli main LOC budget check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
