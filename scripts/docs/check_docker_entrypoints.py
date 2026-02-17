#!/usr/bin/env python3
# Purpose: ensure docs reference make docker entrypoints instead of direct docker build commands.
# Inputs: docs markdown content.
# Outputs: non-zero when forbidden docker commands are present.
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
violations = []
for md in root.joinpath("docs").rglob("*.md"):
    text = md.read_text()
    for m in re.finditer(r"(^|\n)\s*\$\s*docker\s+build\b", text):
        violations.append(f"{md.relative_to(root)}:{text.count(chr(10), 0, m.start())+1}")

if violations:
    print("docs must use make docker-build instead of direct docker build:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("docker entrypoint docs check passed")
