#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ALLOWED = ("artifacts/", "ops/_generated/", "ops/_generated_committed/", "artifacts/evidence/", "docs/_generated/", "scripts/_generated/")
WRITE_RE = re.compile(r"\b(?:>|>>|tee\s+|mkdir\s+-p\s+|cp\s+[^\n]*\s+)([^\s\"']+)")

errors: list[str] = []
for p in sorted((ROOT / "scripts/bin").glob("*")):
    if not p.is_file():
        continue
    rel = p.relative_to(ROOT).as_posix()
    text = p.read_text(encoding="utf-8", errors="ignore")
    for m in WRITE_RE.finditer(text):
        tgt = m.group(1)
        if tgt.startswith("$") or tgt.startswith("/"):
            continue
        if tgt.startswith("."):
            continue
        if any(tgt.startswith(prefix) for prefix in ALLOWED):
            continue
        errors.append(f"{rel}: {tgt}")

if errors:
    print("script write-root policy failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("script write-root policy passed")
