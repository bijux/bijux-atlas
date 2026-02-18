#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OPS_DOCS = ROOT / "docs" / "operations"

headings: dict[str, list[str]] = {}
blocks: dict[str, list[str]] = {}
COMMON_HEADINGS = {
    "what",
    "why",
    "scope",
    "non-goals",
    "contracts",
    "failure modes",
    "how to verify",
    "see also",
    "commands",
    "symptoms",
    "metrics",
    "expected outputs",
    "mitigations",
    "alerts",
    "rollback",
    "postmortem checklist",
    "dashboards",
    "drills",
}

for md in sorted(OPS_DOCS.rglob("*.md")):
    rel = md.relative_to(ROOT).as_posix()
    text = md.read_text(encoding="utf-8", errors="ignore")
    for h in re.findall(r"^##\s+(.+)$", text, flags=re.MULTILINE):
        headings.setdefault(h.strip().lower(), []).append(rel)
    for para in re.split(r"\n\s*\n", text):
        normalized = "\n".join(x.strip() for x in para.splitlines() if x.strip())
        if len(normalized) < 220:
            continue
        key = hashlib.sha256(normalized.encode("utf-8")).hexdigest()
        blocks.setdefault(key, []).append(rel)

errors: list[str] = []
for h, files in headings.items():
    if h in COMMON_HEADINGS:
        continue
    if len(set(files)) > 6:
        errors.append(f"heading appears excessively ({len(set(files))} files): '{h}'")
for _, files in blocks.items():
    if len(set(files)) > 1:
        errors.append(f"duplicated long content block across docs: {', '.join(sorted(set(files)))}")

if errors:
    print("ops docs duplication check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops docs duplication check passed")
