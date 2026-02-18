#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
pattern = re.compile(r"(?:\./)?ops/[^\s`]*\.sh\b")
errors: list[str] = []

for path in sorted((ROOT / "ops").rglob("*.md")):
    if path.name not in {"README.md", "INDEX.md"}:
        continue
    text = path.read_text(encoding="utf-8", errors="ignore")
    for m in pattern.finditer(text):
        snippet = m.group(0)
        errors.append(f"{path.relative_to(ROOT)}: direct script usage in ops docs: {snippet}")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("no direct ops script usage in docs")
