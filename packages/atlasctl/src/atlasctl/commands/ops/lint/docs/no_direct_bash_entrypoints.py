#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
PATTERN = re.compile(r"\bbash\s+ops/.*/scripts/[^\s]+\.sh")


def main() -> int:
    errors: list[str] = []
    for path in sorted((ROOT / "docs").rglob("*.md")):
        for idx, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if PATTERN.search(line):
                errors.append(f"{path.relative_to(ROOT)}:{idx}: direct bash ops script entrypoint is forbidden")
    if errors:
        print("direct bash entrypoint lint failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("direct bash entrypoint lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
