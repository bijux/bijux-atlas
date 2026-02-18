#!/usr/bin/env python3
# Purpose: enforce deterministic docs build settings.
# Inputs: mkdocs.yml and makefiles/docs.mk.
# Outputs: non-zero when non-deterministic settings are configured.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    mkdocs = (ROOT / "mkdocs.yml").read_text(encoding="utf-8")
    docs_mk = (ROOT / "makefiles" / "docs.mk").read_text(encoding="utf-8")
    errors: list[str] = []

    if "enable_creation_date: false" not in mkdocs:
        errors.append("mkdocs.yml must set `enable_creation_date: false`")
    if "fallback_to_build_date: false" not in mkdocs:
        errors.append("mkdocs.yml must set `fallback_to_build_date: false`")
    if "SOURCE_DATE_EPOCH=" not in docs_mk:
        errors.append("makefiles/docs.mk must set SOURCE_DATE_EPOCH for mkdocs build")

    if errors:
        print("docs determinism check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("docs determinism check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
