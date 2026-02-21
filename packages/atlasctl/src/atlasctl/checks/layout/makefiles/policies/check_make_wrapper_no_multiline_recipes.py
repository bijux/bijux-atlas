#!/usr/bin/env python3
# Purpose: ensure wrapper targets use single-line atlasctl delegation recipes.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
WRAPPERS = [
    ROOT / "makefiles" / "dev.mk",
    ROOT / "makefiles" / "ci.mk",
    ROOT / "makefiles" / "docs.mk",
]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)")
MULTILINE_ALLOWLIST = {"makefiles/dev.mk": {"all", "all-all"}}


def main() -> int:
    errors: list[str] = []
    for path in WRAPPERS:
        current = ""
        recipe_lines = 0
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            match = TARGET_RE.match(line)
            if match and not line.startswith("."):
                allow = MULTILINE_ALLOWLIST.get(path.relative_to(ROOT).as_posix(), set())
                expected_lines = 5 if current in allow else 1
                if current and recipe_lines != expected_lines:
                    errors.append(
                        f"{path.relative_to(ROOT)}: target `{current}` must have exactly {expected_lines} recipe line(s) (found {recipe_lines})"
                    )
                current = match.group(1)
                recipe_lines = 0
                continue
            if line.startswith("\t") and current:
                recipe_lines += 1
        allow = MULTILINE_ALLOWLIST.get(path.relative_to(ROOT).as_posix(), set())
        expected_lines = 5 if current in allow else 1
        if current and recipe_lines != expected_lines:
            errors.append(
                f"{path.relative_to(ROOT)}: target `{current}` must have exactly {expected_lines} recipe line(s) (found {recipe_lines})"
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
