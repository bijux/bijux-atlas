#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
DEV_MK = ROOT / "makefiles" / "dev.mk"

FORBIDDEN = {
    "cargo-invocation": re.compile(r"(^|\s)cargo(\s|$)"),
    "python3-invocation": re.compile(r"(^|\s)python3(\s|$)"),
    "rm-cleanup": re.compile(r"(^|\s)rm(\s|$)"),
}


def main() -> int:
    errors: list[str] = []
    for lineno, line in enumerate(DEV_MK.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.startswith("\t"):
            continue
        body = line.strip()
        for name, pattern in FORBIDDEN.items():
            if pattern.search(body):
                errors.append(f"makefiles/dev.mk:{lineno}: forbidden {name} in wrapper-only dev.mk")
    if errors:
        print("dev.mk wrapper purity check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("dev.mk wrapper purity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
