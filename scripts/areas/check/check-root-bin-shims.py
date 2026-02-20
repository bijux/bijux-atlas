#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / "bin"
MAX_LINES = 30
ALLOWED = re.compile(
    r"^(#!|set -euo pipefail|set -eu|ROOT=|PYTHONPATH=|\s*exec python3 -m bijux_atlas_scripts\.cli \"\$@\"|exec \".*/bijux-atlas\" make (explain|graph|help) \"\$@\"|\s*$)"
)


def main() -> int:
    if not BIN.exists():
        return 0
    errors: list[str] = []
    for path in sorted(BIN.iterdir()):
        if path.name == "README.md" or not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        if len(lines) > MAX_LINES:
            errors.append(f"{path.relative_to(ROOT)} exceeds {MAX_LINES} lines")
        for i, line in enumerate(lines, 1):
            if not ALLOWED.match(line):
                errors.append(f"{path.relative_to(ROOT)}:{i}: non-shim logic is forbidden")
                break
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("root bin shim policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
