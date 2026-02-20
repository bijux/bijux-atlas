#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
LEGACY_TARGETS = (
    "ops-stack-up-legacy",
    "ops-stack-down-legacy",
    "ops-check-legacy",
    "ops-smoke-legacy",
)


def main() -> int:
    errs: list[str] = []
    md_files = sorted(DOCS.rglob("*.md"))
    for path in md_files:
        text = path.read_text(encoding="utf-8")
        for target in LEGACY_TARGETS:
            for m in re.finditer(rf"\b{re.escape(target)}\b", text):
                lineno = text.count("\n", 0, m.start()) + 1
                errs.append(f"{path.relative_to(ROOT)}:{lineno}: legacy target reference `{target}`")

    if errs:
        print("legacy targets in docs check failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("legacy targets in docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
