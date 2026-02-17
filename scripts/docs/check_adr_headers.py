#!/usr/bin/env python3
# Purpose: enforce ADR filename/header style.
# Inputs: docs/adrs/ADR-*.md
# Outputs: non-zero exit on mismatch.
from __future__ import annotations

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
errors: list[str] = []
acronyms = {"ADR", "API", "SSOT", "CLI", "CI", "SQL", "SQLITE", "K8S"}
for path in sorted((ROOT / "docs" / "adrs").glob("ADR-*.md")):
    if path.name == "INDEX.md":
        continue
    m = re.match(r"ADR-(\d{4})-([a-z0-9-]+)\.md$", path.name)
    if not m:
        errors.append(f"invalid ADR filename: {path}")
        continue
    num = m.group(1)
    first = path.read_text(encoding="utf-8").splitlines()[0].strip()
    prefix = f"# ADR-{num}: "
    if not first.startswith(prefix):
        errors.append(f"header mismatch in {path}: missing `{prefix}` prefix")
        continue
    title = first[len(prefix):].strip()
    if not title:
        errors.append(f"header mismatch in {path}: missing ADR title text")
        continue
    for word in re.findall(r"[A-Za-z0-9]+", title):
        if word.upper() in acronyms:
            continue
        if not word[0].isupper():
            errors.append(f"header mismatch in {path}: non-title-case word `{word}`")
            break

if errors:
    print("ADR header check failed:")
    for err in errors:
        print(f"- {err}")
    raise SystemExit(1)
print("ADR header check passed")
