#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CARGO_MK = ROOT / "makefiles" / "cargo.mk"

FORBIDDEN = {
    "cargo-invocation": re.compile(r"(^|\s)cargo(\s|$)"),
    "python3-invocation": re.compile(r"(^|\s)python3(\s|$)"),
    "rm-cleanup": re.compile(r"(^|\s)rm(\s|$)"),
}


def main() -> int:
    errors: list[str] = []
    for lineno, line in enumerate(CARGO_MK.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.startswith("\t"):
            continue
        body = line.strip()
        for name, pattern in FORBIDDEN.items():
            if pattern.search(body):
                errors.append(f"makefiles/cargo.mk:{lineno}: forbidden {name} in wrapper-only cargo.mk")
    if errors:
        print("cargo.mk wrapper purity check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("cargo.mk wrapper purity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

