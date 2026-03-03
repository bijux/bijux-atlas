#!/usr/bin/env python3
"""Validate the documented navigation constraints in mkdocs.yml."""

from __future__ import annotations

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MKDOCS_PATH = REPO_ROOT / "mkdocs.yml"


def require_exactly_one(text: str, marker: str) -> None:
    count = text.count(marker)
    if count != 1:
        raise SystemExit(f"expected exactly one '{marker}' entry, found {count}")


def main() -> None:
    text = MKDOCS_PATH.read_text()
    require_exactly_one(text, "- Start Here: start-here.md")
    require_exactly_one(text, "- Governance: _internal/governance/index.md")


if __name__ == "__main__":
    main()
