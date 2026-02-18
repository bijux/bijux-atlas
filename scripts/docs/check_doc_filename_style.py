#!/usr/bin/env python3
# Purpose: enforce docs filename style: kebab-case, plus explicit canonical exceptions.
# Inputs: docs/**/*.md
# Outputs: non-zero on style violations.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

KEBAB = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
INDEX = re.compile(r"^INDEX\.md$")
ADR = re.compile(r"^ADR-\d{4}-[a-z0-9-]+\.md$")
SCREAM = re.compile(r"^[A-Z0-9_]+\.md$")
EXCEPTIONS = {
    "docs/STYLE.md",
    "docs/contracts/README.md",
}


def allowed(path: Path) -> bool:
    rel = str(path.relative_to(ROOT))
    if rel in EXCEPTIONS:
        return True
    name = path.name
    if KEBAB.match(name) or INDEX.match(name) or ADR.match(name):
        return True
    if "docs/_style/" in str(path.relative_to(ROOT)) and SCREAM.match(name):
        return True
    if "docs/_generated/contracts/" in str(path.relative_to(ROOT)) and SCREAM.match(name):
        return True
    return False


def main() -> int:
    bad: list[str] = []
    for path in sorted(DOCS.rglob("*.md")):
        if not allowed(path):
            bad.append(str(path.relative_to(ROOT)))
    if bad:
        print("doc filename style check failed:", file=sys.stderr)
        for item in bad:
            print(f"- {item}", file=sys.stderr)
        return 1
    print("doc filename style check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
