#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    errs: list[str] = []

    scripts_dir = ROOT / "scripts"
    if scripts_dir.exists():
        errs.append("scripts/ directory still exists; final SSOT migration requires removal")

    pattern = re.compile(r"python(3)?\s+\.?/?scripts/")
    for mk in sorted((ROOT / "makefiles").glob("*.mk")):
        for idx, line in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if pattern.search(line):
                errs.append(f"{mk.relative_to(ROOT)}:{idx}: forbidden non-SSOT script invocation")

    if errs:
        print("scripts SSOT final gate failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("scripts SSOT final gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
