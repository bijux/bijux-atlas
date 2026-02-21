#!/usr/bin/env python3
# Purpose: enforce docs.mk wrapper target count budget.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
DOCS_MK = ROOT / "makefiles" / "docs.mk"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
MAX_TARGETS = 25


def _count_targets() -> int:
    text = DOCS_MK.read_text(encoding="utf-8")
    targets = [
        name
        for name in TARGET_RE.findall(text)
        if not name.startswith(".") and not name.startswith("_") and not name.startswith("internal/")
    ]
    return len(targets)


def main() -> int:
    count = _count_targets()
    if count > MAX_TARGETS:
        print(
            f"docs.mk target budget check failed: {count} > {MAX_TARGETS}",
            file=sys.stderr,
        )
        return 1
    print(f"docs.mk target budget check passed: {count} <= {MAX_TARGETS}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
