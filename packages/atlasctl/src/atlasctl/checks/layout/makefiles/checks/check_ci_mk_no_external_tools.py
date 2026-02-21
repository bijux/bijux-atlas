#!/usr/bin/env python3
# Purpose: ensure ci.mk recipes only delegate via atlasctl wrappers.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CI_MK = ROOT / "makefiles" / "ci.mk"
ATLASCTL_RE = re.compile(r"^\t@\.\/bin\/atlasctl\b")


def main() -> int:
    errors: list[str] = []
    for lineno, line in enumerate(CI_MK.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.startswith("\t"):
            continue
        if not ATLASCTL_RE.match(line):
            errors.append(f"makefiles/ci.mk:{lineno}: recipe must delegate with ./bin/atlasctl only")
    if errors:
        print("ci.mk external-tool guard failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ci.mk external-tool guard passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
