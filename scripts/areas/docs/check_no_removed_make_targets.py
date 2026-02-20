#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCAN_DIRS = [ROOT / "docs", ROOT / "makefiles" / "README.md"]
REMOVED = {"docker", "chart"}
PATTERNS = [re.compile(rf"\bmake\s+{re.escape(t)}(?=\s|$|`|,)") for t in sorted(REMOVED)]


def iter_files() -> list[Path]:
    out: list[Path] = []
    for item in SCAN_DIRS:
        if item.is_file():
            out.append(item)
        elif item.is_dir():
            out.extend(sorted(item.rglob("*.md")))
    return out


def main() -> int:
    violations: list[str] = []
    for path in iter_files():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for i, line in enumerate(text.splitlines(), start=1):
            if "make " not in line:
                continue
            for pat in PATTERNS:
                if pat.search(line):
                    violations.append(f"{rel}:{i}: removed public target reference: {pat.pattern}")
                    break
    if violations:
        print("removed make target docs check failed:", file=sys.stderr)
        for v in violations:
            print(f"- {v}", file=sys.stderr)
        return 1
    print("removed make target docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
