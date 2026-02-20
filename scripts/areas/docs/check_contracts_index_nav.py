#!/usr/bin/env python3
# Purpose: ensure generated contracts index includes every CONTRACT.md and is present in docs nav.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MKDOCS = ROOT / "mkdocs.yml"
INDEX = ROOT / "docs" / "_generated" / "contracts-index.md"
SCAN_ROOTS = [
    ROOT / "ops",
    ROOT / "configs",
    ROOT / "makefiles",
    ROOT / "docker",
]


def main() -> int:
    errors: list[str] = []
    mkdocs = MKDOCS.read_text(encoding="utf-8")
    if "_generated/contracts-index.md" not in mkdocs:
        errors.append("mkdocs.yml is missing nav entry for docs/_generated/contracts-index.md")

    if not INDEX.exists():
        errors.append("missing docs/_generated/contracts-index.md; run docs generator")
    else:
        index_text = INDEX.read_text(encoding="utf-8")
        contracts: list[Path] = []
        for scan_root in SCAN_ROOTS:
            contracts.extend(scan_root.glob("**/CONTRACT.md"))
        contracts = sorted(contracts)
        for path in contracts:
            rel = path.relative_to(ROOT).as_posix()
            if f"`{rel}`" not in index_text:
                errors.append(f"contracts index missing `{rel}`")

    if errors:
        print("contracts index/nav check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("contracts index/nav check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
