#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CI_MK = ROOT / "makefiles" / "ci.mk"
MAX_LINES = 120


def main() -> int:
    lines = len(CI_MK.read_text(encoding="utf-8").splitlines())
    if lines > MAX_LINES:
        print(
            f"ci.mk size budget exceeded: {lines} > {MAX_LINES} (move logic into atlasctl dev ci subcommands)",
            file=sys.stderr,
        )
        return 1
    print(f"ci.mk size budget check passed: {lines}/{MAX_LINES}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
