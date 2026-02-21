#!/usr/bin/env python3
# Purpose: enforce wrapper makefiles delegate through atlasctl only.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
WRAPPERS = [
    ROOT / "makefiles" / "dev.mk",
    ROOT / "makefiles" / "ci.mk",
    ROOT / "makefiles" / "docs.mk",
]
ATLASCTL_RE = re.compile(r"^\t@\.\/bin\/atlasctl\b")


def main() -> int:
    errors: list[str] = []
    for path in WRAPPERS:
        text = path.read_text(encoding="utf-8")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if not line.startswith("\t"):
                continue
            if not line.strip():
                continue
            if not ATLASCTL_RE.match(line):
                errors.append(f"{path.relative_to(ROOT)}:{lineno}: wrapper recipe must delegate via ./bin/atlasctl")

    if errors:
        print("make delegation-only check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("make delegation-only check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
