#!/usr/bin/env python3
# Purpose: ensure every public target in SSOT appears in generated docs and nav.
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGETS = ROOT / "makefiles" / "targets.json"
DOC = ROOT / "docs" / "_generated" / "make-targets.md"
MKDOCS = ROOT / "mkdocs.yml"


def main() -> int:
    errors: list[str] = []
    if "_generated/make-targets.md" not in MKDOCS.read_text(encoding="utf-8"):
        errors.append("mkdocs.yml missing nav entry for docs/_generated/make-targets.md")
    targets = json.loads(TARGETS.read_text(encoding="utf-8")).get("targets", [])
    doc_text = DOC.read_text(encoding="utf-8")
    for target in targets:
        name = target.get("name", "")
        if not name:
            continue
        if f"`{name}`" not in doc_text:
            errors.append(f"docs/_generated/make-targets.md missing `{name}`")

    if errors:
        print("public target docs section check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("public target docs section check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
