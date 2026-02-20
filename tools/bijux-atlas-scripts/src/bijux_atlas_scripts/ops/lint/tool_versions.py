#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
scopes = [ROOT / "ops", ROOT / "makefiles"]
version_re = re.compile(r"\b(?:kind|kubectl|helm|k6)\b[^\n]{0,40}\bv?\d+\.\d+(?:\.\d+)?\b", re.IGNORECASE)
ignore = (
    "configs/ops/tool-versions.json",
    "ops/stack/versions.json",
)
errors: list[str] = []

for scope in scopes:
    for path in scope.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT).as_posix()
        if rel in ignore:
            continue
        if any(rel.startswith(prefix.rstrip("*")) for prefix in ("ops/_artifacts/", "ops/_generated/")):
            continue
        if path.suffix not in {".sh", ".py", ".mk"}:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "tool-versions.json" in text:
            continue
        if version_re.search(text):
            errors.append(f"possible floating tool version in {rel}")

if errors:
    for e in sorted(set(errors)):
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("no floating tool versions detected")
