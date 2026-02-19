#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOOL_RE = re.compile(r"\b(kubectl|helm|kind|k6)\b")
GUARDS = ("check_tool_versions.py", "ops_version_guard", "scripts/layout/check_tool_versions.py")

errors: list[str] = []
scan_dirs = [ROOT / "scripts/bin", ROOT / "scripts/check", ROOT / "scripts/ci"]
for scan_dir in scan_dirs:
    if not scan_dir.exists():
        continue
    for p in sorted(scan_dir.rglob("*.sh")):
        rel = p.relative_to(ROOT).as_posix()
        text = p.read_text(encoding="utf-8", errors="ignore")
        if not TOOL_RE.search(text):
            continue
        if any(g in text for g in GUARDS):
            continue
        errors.append(rel)

if errors:
    print("scripts using kubectl/helm/kind/k6 without version guard:", file=sys.stderr)
    for rel in errors:
        print(f"- {rel}", file=sys.stderr)
    raise SystemExit(1)

print("script tool guard check passed")
