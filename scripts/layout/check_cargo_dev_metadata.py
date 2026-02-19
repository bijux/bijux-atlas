#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CARGO_DEV = ROOT / "makefiles" / "cargo-dev.mk"

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
DEV_META_RE = re.compile(r"^([A-Za-z0-9_./-]+):.*##\s*DEV_ONLY=1\s*$", re.M)


def main() -> int:
    text = CARGO_DEV.read_text(encoding="utf-8")
    targets = [t for t in TARGET_RE.findall(text) if not t.startswith(".")]
    meta = set(DEV_META_RE.findall(text))

    errors: list[str] = []
    for t in targets:
        if t not in meta:
            errors.append(f"missing DEV_ONLY=1 metadata for target: {t}")

    if errors:
        print("cargo-dev metadata check failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("cargo-dev metadata check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
