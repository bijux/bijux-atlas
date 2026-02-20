#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
DOCS = [ROOT / "docs", ROOT / "makefiles" / "README.md", ROOT / "README.md"]
PATTERNS = [
    re.compile(r"\bmake\s+(internal/[A-Za-z0-9_./-]+)\b"),
    re.compile(r"\bmake\s+(_[A-Za-z0-9_./-]+)\b"),
]


def main() -> int:
    errors: list[str] = []
    for path in DOCS:
        files = [path] if path.is_file() else list(path.rglob("*.md"))
        for file in files:
            text = file.read_text(encoding="utf-8")
            for pattern in PATTERNS:
                for match in pattern.finditer(text):
                    errors.append(f"{file.relative_to(ROOT)} references internal target: {match.group(1)}")

    if errors:
        print("internal target docs reference check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("internal target docs reference check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
