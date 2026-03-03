#!/usr/bin/env python3
"""Synchronize MkDocs redirect_maps from docs/redirects.json."""

from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MKDOCS_PATH = REPO_ROOT / "mkdocs.yml"
REDIRECTS_PATH = REPO_ROOT / "docs" / "redirects.json"
START = "      # redirect_maps generated from docs/redirects.json; run scripts/docs/sync_redirects.py"
END = "      # end generated redirect_maps"


def is_mkdocs_page(path: str) -> bool:
    return path.endswith(".md")


def to_mkdocs_key(path: str) -> str:
    if not path.startswith("docs/"):
        raise SystemExit(f"redirect key must start with docs/: {path}")
    return path[len("docs/") :]


def to_mkdocs_value(path: str) -> str:
    if not path.startswith("docs/"):
        raise SystemExit(f"redirect value must start with docs/: {path}")
    return path[len("docs/") :]


def render_block(mapping: dict[str, str]) -> str:
    filtered = {
        key: value
        for key, value in mapping.items()
        if is_mkdocs_page(key) and is_mkdocs_page(value)
    }
    lines = [
        START,
        "      redirect_maps:",
    ]
    for key in sorted(filtered):
        lines.append(f"        {to_mkdocs_key(key)}: {to_mkdocs_value(filtered[key])}")
    lines.append(END)
    return "\n".join(lines)


def main() -> None:
    mapping = json.loads(REDIRECTS_PATH.read_text())
    mkdocs_text = MKDOCS_PATH.read_text()
    try:
        start_index = mkdocs_text.index(START)
        end_index = mkdocs_text.index(END)
    except ValueError as exc:
        raise SystemExit(f"mkdocs redirect markers missing: {exc}") from exc
    end_index = mkdocs_text.index("\n", end_index)
    new_text = mkdocs_text[:start_index] + render_block(mapping) + mkdocs_text[end_index:]
    MKDOCS_PATH.write_text(new_text)


if __name__ == "__main__":
    main()
