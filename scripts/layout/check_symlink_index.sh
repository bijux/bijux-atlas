#!/usr/bin/env python3
# Purpose: ensure every root-level symlink is documented in docs/development/symlinks.md.
# Inputs: filesystem symlinks at repo root and symlink index doc.
# Outputs: non-zero on undocumented symlink.
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
doc = root / "docs" / "development" / "symlinks.md"
if not doc.exists():
    print(f"missing symlink index: {doc}", file=sys.stderr)
    raise SystemExit(1)

text = doc.read_text()
documented = set(re.findall(r"- `([^`]+)` -> `[^`]+`", text))

root_symlinks = {p.name for p in root.iterdir() if p.is_symlink()}
missing = sorted(root_symlinks - documented)
if missing:
    print("undocumented root symlinks:", file=sys.stderr)
    for item in missing:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

print("symlink index check passed")
