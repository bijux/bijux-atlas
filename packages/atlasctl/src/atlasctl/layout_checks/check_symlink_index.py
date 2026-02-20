#!/usr/bin/env python3
# Purpose: ensure every root-level symlink is documented and allowlisted.
# Inputs: root symlinks, docs/development/symlinks.md, configs/repo/symlink-allowlist.json.
# Outputs: non-zero on undocumented or unallowlisted root symlinks.
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
doc = root / "docs" / "development" / "symlinks.md"
allowlist = root / "configs" / "repo" / "symlink-allowlist.json"
if not doc.exists():
    print(f"missing symlink index: {doc}", file=sys.stderr)
    raise SystemExit(1)
if not allowlist.exists():
    print(f"missing symlink allowlist: {allowlist}", file=sys.stderr)
    raise SystemExit(1)

text = doc.read_text(encoding="utf-8")
documented = set(re.findall(r"- `([^`]+)` -> `[^`]+`", text))
allow_root = set(json.loads(allowlist.read_text(encoding="utf-8")).get("root", {}).keys())

root_symlinks = {p.name for p in root.iterdir() if p.is_symlink()}
missing_docs = sorted(root_symlinks - documented)
if missing_docs:
    print("undocumented root symlinks:", file=sys.stderr)
    for item in missing_docs:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

missing_allowlist = sorted(root_symlinks - allow_root)
if missing_allowlist:
    print("root symlinks missing allowlist entries:", file=sys.stderr)
    for item in missing_allowlist:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

print("symlink index check passed")
