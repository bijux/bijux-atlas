#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
errors: list[str] = []
for p in sorted((ROOT / "scripts/bin").glob("bijux-atlas-*")):
    if not p.is_file():
        continue
    if p.name == "bijux-atlas-dev":
        # Informational index wrapper; no command execution path.
        continue
    text = p.read_text(encoding="utf-8", errors="ignore")
    if "python3 -m bijux_atlas_scripts.cli" in text:
        continue
    if '"error_code"' not in text and "err(" not in text:
        errors.append(f"{p.relative_to(ROOT)} must emit structured JSON error_code or delegate to atlasctl")

if errors:
    print("structured error contract failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("structured error contract passed")
