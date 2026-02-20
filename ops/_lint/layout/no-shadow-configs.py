#!/usr/bin/env python3
from __future__ import annotations

import json
import fnmatch
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SOURCES = ROOT / "configs/ops/config-sources.json"
errors: list[str] = []

rules = json.loads(SOURCES.read_text(encoding="utf-8"))
for concept, cfg in rules.get("concepts", {}).items():
    canonical = cfg["canonical"]
    if not (ROOT / canonical).exists():
        errors.append(f"{concept}: missing canonical config {canonical}")
        continue
    allow = set(cfg.get("allow", []))
    for pattern in cfg.get("forbid_globs", []):
        for p in ROOT.glob(pattern):
            if not p.exists() or not p.is_file():
                continue
            rel = p.relative_to(ROOT).as_posix()
            if (
                rel.startswith("ops/_generated/")
                or rel.startswith("ops/_generated_committed/")
                or rel.startswith("ops/_artifacts/")
                or rel.startswith("ops/_evidence/")
                or rel.startswith("artifacts/evidence/")
                or rel.startswith("artifacts/")
            ):
                continue
            if rel == canonical or rel in allow:
                continue
            if fnmatch.fnmatch(rel, pattern):
                errors.append(f"{concept}: shadow config source detected: {rel}")

if errors:
    for e in sorted(set(errors)):
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("no shadow ops config sources detected")
