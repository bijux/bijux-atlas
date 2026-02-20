#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
MAKEFILES = sorted((ROOT / "makefiles").glob("*.mk"))
CARGO_RE = re.compile(r"(^|\s)cargo(\s|$)")
LEGACY = ROOT / "configs" / "ops" / "cargo-invocation-legacy.txt"

ALLOWED_FILES = {"cargo.mk", "cargo-dev.mk"}


def main() -> int:
    errors: list[str] = []
    legacy = {
        line.strip()
        for line in LEGACY.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }
    for path in MAKEFILES:
        rel = path.relative_to(ROOT)
        text = path.read_text(encoding="utf-8")
        for i, line in enumerate(text.splitlines(), start=1):
            if not line.startswith("\t"):
                continue
            if not CARGO_RE.search(line):
                continue
            if path.name in ALLOWED_FILES:
                continue
            key = f"{rel}:{i}"
            if key in legacy:
                continue
            errors.append(f"{rel}:{i}: new cargo invocation outside makefiles/cargo*.mk")

    if errors:
        print("cargo invocation scope check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("cargo invocation scope check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
