#!/usr/bin/env python3
# Purpose: ensure wrapper targets use single-line atlasctl delegation recipes.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
WRAPPERS = [
    ROOT / "makefiles" / "dev.mk",
]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)")


def main() -> int:
    errors: list[str] = []
    for path in WRAPPERS:
        current = ""
        recipe_lines = 0
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            match = TARGET_RE.match(line)
            if match and not line.startswith("."):
                if current and recipe_lines != 1:
                    errors.append(
                        f"{path.relative_to(ROOT)}: target `{current}` must have exactly one recipe line (found {recipe_lines})"
                    )
                current = match.group(1)
                recipe_lines = 0
                continue
            if line.startswith("\t") and current:
                recipe_lines += 1
        if current and recipe_lines != 1:
            errors.append(
                f"{path.relative_to(ROOT)}: target `{current}` must have exactly one recipe line (found {recipe_lines})"
            )

    if errors:
        print("make wrapper multi-line recipe check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("make wrapper multi-line recipe check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
