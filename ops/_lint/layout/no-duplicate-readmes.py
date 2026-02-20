#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
readmes = sorted((ROOT / "ops").rglob("README.md"))

def normalize(text: str) -> str:
    text = re.sub(r"\s+", " ", text.strip().lower())
    return text

by_digest: dict[str, list[Path]] = {}
for path in readmes:
    digest = hashlib.sha256(normalize(path.read_text(encoding="utf-8", errors="ignore")).encode()).hexdigest()
    by_digest.setdefault(digest, []).append(path)

errors = [paths for paths in by_digest.values() if len(paths) > 1]
if errors:
    for dup in errors:
        rels = ", ".join(str(p.relative_to(ROOT)) for p in dup)
        print(f"duplicate README content: {rels}", file=sys.stderr)
    raise SystemExit(1)

print("no duplicate README content")
