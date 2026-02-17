#!/usr/bin/env python3
"""
Purpose: Validate doc status frontmatter and ban draft pages.
Inputs: docs/**/*.md
Outputs: docs/_generated/doc-status.md summary.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
OUT = DOCS / "_generated" / "doc-status.md"
ALLOWED = {"active", "frozen", "draft"}


def read_status(path: Path) -> str | None:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end == -1:
        return None
    frontmatter = text[4:end]
    for line in frontmatter.splitlines():
        m = re.match(r"status:\s*([a-zA-Z-]+)\s*$", line.strip())
        if m:
            return m.group(1).lower()
    return None


def badge(status: str) -> str:
    mapping = {
        "active": "![active](https://img.shields.io/badge/status-active-brightgreen)",
        "frozen": "![frozen](https://img.shields.io/badge/status-frozen-blue)",
        "draft": "![draft](https://img.shields.io/badge/status-draft-lightgrey)",
    }
    return mapping[status]


def main() -> int:
    drafted: list[str] = []
    invalid: list[str] = []
    rows: list[tuple[str, str]] = []

    for path in sorted(DOCS.rglob("*.md")):
        rp = path.relative_to(DOCS).as_posix()
        if rp.startswith("_generated/"):
            continue
        status = read_status(path)
        if status is None:
            continue
        if status not in ALLOWED:
            invalid.append(f"{rp}: {status}")
            continue
        rows.append((rp, status))
        if status == "draft":
            drafted.append(rp)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    lines = ["# Document Status", "", "## What", "Status summary generated from document frontmatter.", "", "## Contracts", "- Allowed statuses: `active`, `frozen`, `draft`.", "- `draft` is forbidden on default branch.", "", "## Pages", "", "| Page | Status |", "|---|---|"]
    for rp, status in rows:
        lines.append(f"| `{rp}` | {badge(status)} `{status}` |")
    if not rows:
        lines.append("| (none) | n/a |")
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if invalid:
        print("invalid doc status values:")
        for item in invalid:
            print(f"- {item}")
        return 1
    if drafted:
        print("draft docs are not allowed:")
        for item in drafted:
            print(f"- {item}")
        return 1
    print("doc status lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
