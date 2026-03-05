#!/usr/bin/env python3
"""Generate Python client documentation index."""

from __future__ import annotations

import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"

entries = [
    ("Architecture", "architecture.md"),
    ("API Reference", "api-reference.md"),
    ("Quickstart", "quickstart.md"),
    ("Troubleshooting", "troubleshooting.md"),
    ("Version Compatibility Matrix", "version-compatibility-matrix.md"),
]

content = ["# Python Client Documentation", ""]
for title, rel in entries:
    content.append(f"- [{title}]({rel})")

(DOCS / "index.md").write_text("\n".join(content) + "\n", encoding="utf-8")
print("generated", DOCS / "index.md")
