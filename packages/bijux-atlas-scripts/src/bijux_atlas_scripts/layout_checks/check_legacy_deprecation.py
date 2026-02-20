#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
FORBIDDEN_RE = re.compile(r"(^|/)legacy($|-)")


def main() -> int:
    errs: list[str] = []
    for mk in sorted((ROOT / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        for target in TARGET_RE.findall(text):
            if target.startswith("."):
                continue
            if FORBIDDEN_RE.search(target):
                errs.append(f"{mk.relative_to(ROOT)} contains forbidden legacy target: {target}")

    if errs:
        print("legacy target contract failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("legacy target contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
