#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
INDEX = ROOT / "makefiles" / "INDEX.md"
MAX_FILES = 14


def main() -> int:
    mk_files = sorted((ROOT / "makefiles").glob("*.mk"))
    count = len(mk_files)
    if count <= MAX_FILES:
        print("makefiles index drift check passed")
        return 0

    if not INDEX.exists():
        print("makefiles index drift check failed", file=sys.stderr)
        print(f"- makefiles count {count} exceeds {MAX_FILES} and makefiles/INDEX.md is missing", file=sys.stderr)
        return 1

    text = INDEX.read_text(encoding="utf-8")
    m = re.search(r"File count:\s*(\d+)", text)
    if not m:
        print("makefiles index drift check failed", file=sys.stderr)
        print("- makefiles/INDEX.md missing 'File count: <n>'", file=sys.stderr)
        return 1
    declared = int(m.group(1))
    if declared != count:
        print("makefiles index drift check failed", file=sys.stderr)
        print(f"- makefiles/INDEX.md File count mismatch: declared {declared}, actual {count}", file=sys.stderr)
        return 1

    print("makefiles index drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
