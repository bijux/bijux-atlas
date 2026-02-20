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
    if "python3 -m bijux_atlas_scripts.cli" not in text:
        errors.append("makefiles/python.mk must invoke atlasctl via python -m bijux_atlas_scripts.cli")

    docs_text = (ROOT / "docs/development/tooling/bijux-atlas-scripts.md").read_text(
        encoding="utf-8", errors="ignore"
    )
    if re.search(r"scripts/bin/bijux-atlas-scripts", docs_text):
        errors.append("docs still reference scripts/bin/bijux-atlas-scripts")
    if "atlasctl" not in docs_text:
        errors.append("docs/development/tooling/bijux-atlas-scripts.md must reference atlasctl")

    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print("invocation parity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
