#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    errors: list[str] = []
    py_mk = ROOT / "makefiles/python.mk"
    text = py_mk.read_text(encoding="utf-8")
    if "ATLAS_SCRIPTS ?= ./bin/bijux-atlas" not in text:
        errors.append("makefiles/python.mk must set ATLAS_SCRIPTS to ./bin/bijux-atlas")

    docs_text = (ROOT / "docs/development/tooling/bijux-atlas-scripts.md").read_text(
        encoding="utf-8", errors="ignore"
    )
    if re.search(r"scripts/bin/bijux-atlas-scripts", docs_text):
        errors.append("docs still reference scripts/bin/bijux-atlas-scripts")
    if "bijux-atlas" not in docs_text:
        errors.append("docs/development/tooling/bijux-atlas-scripts.md must reference bijux-atlas")

    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print("invocation parity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
