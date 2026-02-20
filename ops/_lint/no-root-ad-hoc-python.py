#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

ALLOWED_PREFIXES = (
    "packages/",
    "ops/",
    "scripts/",
    "tools/",
)

SKIP_PREFIXES = (
    "artifacts/",
    ".venv/",
    ".mypy_cache/",
    ".ruff_cache/",
    ".hypothesis/",
    "target/",
)


def main() -> int:
    errors: list[str] = []
    for path in ROOT.rglob("*.py"):
        if "/.git/" in str(path):
            continue
        rel = path.relative_to(ROOT).as_posix()
        if "/__pycache__/" in rel:
            continue
        if rel.startswith(SKIP_PREFIXES):
            continue
        if rel.startswith(ALLOWED_PREFIXES):
            continue
        errors.append(rel)
    if errors:
        print("root ad-hoc python lint failed:", file=sys.stderr)
        for rel in errors:
            print(f"- {rel}", file=sys.stderr)
        return 1
    print("root ad-hoc python lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
