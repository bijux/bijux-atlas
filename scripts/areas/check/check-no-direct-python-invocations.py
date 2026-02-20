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

DIRECT_PY_RE = re.compile(r"\bpython3?\s+([^\s`]+\.py)\b")
PY_SCRIPTS_RE = re.compile(r"\bpython3?\s+scripts/[^\s`]+\.py\b")
ALLOWED_MAKE_RE = re.compile(r"\bpython3?\s+-m\s+bijux_atlas_scripts(?:\b|$)")


def _scan_file(path: Path, kind: str, errors: list[str]) -> None:
    rel = path.relative_to(ROOT).as_posix()
    text = path.read_text(encoding="utf-8", errors="ignore")
    for lineno, line in enumerate(text.splitlines(), start=1):
        if PY_SCRIPTS_RE.search(line):
            if find_matching_exception("python_scripts_path", rel, line) is None:
                errors.append(f"{rel}:{lineno}: direct `python scripts/*.py` invocation is forbidden")

        if kind == "docs":
            if DIRECT_PY_RE.search(line):
                if find_matching_exception("docs_direct_python", rel, line) is None:
                    errors.append(
                        f"{rel}:{lineno}: docs must reference `bijux-atlas-scripts`, not direct python execution"
                    )
        if kind == "makefiles":
            if DIRECT_PY_RE.search(line) and not ALLOWED_MAKE_RE.search(line):
                if find_matching_exception("makefiles_direct_python", rel, line) is None:
                    errors.append(
                        f"{rel}:{lineno}: makefiles must use `bijux-atlas-scripts` or `python -m bijux_atlas_scripts...`"
                    )


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
        print("direct python invocation policy check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("direct python invocation policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
