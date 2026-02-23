#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
SRC = ROOT / "packages/atlasctl/src/atlasctl"


def main() -> int:
    runners = sorted(p.relative_to(ROOT).as_posix() for p in SRC.rglob("runner.py"))
    expected = ["packages/atlasctl/src/atlasctl/checks/engine/runner.py"]
    if runners != expected:
        print(f"runner surface drift: expected {expected}, got {runners}")
        return 1
    print("single checks engine runner OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
