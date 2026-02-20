#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
errors: list[str] = []

script_cmd = re.compile(r"^\s*(\./ops/|bash\s+ops/|sh\s+ops/|python3\s+ops/)")
make_cmd = re.compile(r"\bmake\s+[a-zA-Z0-9_.-]+")

for md in sorted((ROOT / "ops").rglob("README.md")):
    text = md.read_text(encoding="utf-8", errors="ignore")
    has_make = bool(make_cmd.search(text))
    for line_no, line in enumerate(text.splitlines(), start=1):
        if script_cmd.search(line):
            errors.append(f"{md.relative_to(ROOT)}:{line_no}: raw script run instruction found")
    if not has_make:
        errors.append(f"{md.relative_to(ROOT)}: missing make target instruction")

if errors:
    print("ops README make-only contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops README make-only contract passed")
