#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCRIPT_REF = re.compile(r"(?:\./)?ops/(?!run/)[^\s`\"']*\.sh\b")
LEGACY_REF = re.compile(r"(?:\./)?ops/[^\s`\"']*(?:legacy|_legacy)[^\s`\"']*")
TARGET_LEGACY = re.compile(r"\b(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy)\b")
errors: list[str] = []

SCAN_FILES = [ROOT / "makefiles" / "root.mk", ROOT / "makefiles" / "ci.mk"]
SCAN_DIRS = [ROOT / ".github/workflows"]

for path in SCAN_FILES:
    if not path.exists():
        continue
    rel = path.relative_to(ROOT)
    text = path.read_text(encoding="utf-8", errors="ignore")
    for m in SCRIPT_REF.finditer(text):
        errors.append(f"{rel}: direct non-run ops script reference: {m.group(0)}")
    for m in LEGACY_REF.finditer(text):
        errors.append(f"{rel}: legacy ops path reference: {m.group(0)}")
    for m in TARGET_LEGACY.finditer(text):
        errors.append(f"{rel}: legacy target reference: {m.group(0)}")

for base in SCAN_DIRS:
    if not base.exists():
        continue
    for path in sorted(base.rglob("*")):
        if path.is_dir():
            continue
        if any(part in {"_generated", "_generated_committed", "_evidence", "_artifacts"} for part in path.parts):
            continue
        if path.suffix not in {".md", ".mk", ".sh", ".py", ".yml", ".yaml", ".json"}:
            continue
        rel = path.relative_to(ROOT)
        text = path.read_text(encoding="utf-8", errors="ignore")
        for m in SCRIPT_REF.finditer(text):
            errors.append(f"{rel}: direct non-run ops script reference: {m.group(0)}")
        for m in LEGACY_REF.finditer(text):
            errors.append(f"{rel}: legacy ops path reference: {m.group(0)}")
        for m in TARGET_LEGACY.finditer(text):
            errors.append(f"{rel}: legacy target reference: {m.group(0)}")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("no direct non-run ops script usage or legacy path references")
