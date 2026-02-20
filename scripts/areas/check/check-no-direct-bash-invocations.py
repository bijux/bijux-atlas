#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

from python_migration_exceptions import find_matching_exception

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
MAKEFILES = ROOT / "makefiles"
MAKEFILE = ROOT / "Makefile"

BASH_SCRIPTS_RE = re.compile(r"\bbash\s+([^\s`]*scripts/[^\s`]+)\b")


def _scan_file(path: Path, kind: str, errors: list[str]) -> None:
    rel = path.relative_to(ROOT).as_posix()
    text = path.read_text(encoding="utf-8", errors="ignore")
    for lineno, line in enumerate(text.splitlines(), start=1):
        if not BASH_SCRIPTS_RE.search(line):
            continue
        exc_kind = "docs_direct_bash" if kind == "docs" else "makefiles_direct_bash"
        if find_matching_exception(exc_kind, rel, line) is None:
            errors.append(f"{rel}:{lineno}: direct `bash ...scripts/...` invocation is forbidden")


def main() -> int:
    errors: list[str] = []
    for path in DOCS.rglob("*.md"):
        if "docs/_generated/" in path.as_posix():
            continue
        _scan_file(path, "docs", errors)

    for path in MAKEFILES.glob("*.mk"):
        _scan_file(path, "makefiles", errors)
    _scan_file(MAKEFILE, "makefiles", errors)

    if errors:
        print("direct bash invocation policy check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("direct bash invocation policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
