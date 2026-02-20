#!/usr/bin/env python3
from __future__ import annotations

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ROOT_MK = ROOT / "makefiles" / "root.mk"


def main() -> int:
    max_lines = int(os.getenv("ROOT_MK_MAX_LINES", "620"))
    lines = sum(1 for _ in ROOT_MK.open("r", encoding="utf-8"))
    if lines > max_lines:
        print(
            f"root.mk size budget exceeded: {lines} > {max_lines} (refactor into dedicated makefiles/*.mk)",
            file=sys.stderr,
        )
        return 1
    print(f"root.mk size budget check passed: {lines}/{max_lines}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
