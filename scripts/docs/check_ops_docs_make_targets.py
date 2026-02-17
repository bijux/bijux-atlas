#!/usr/bin/env python3
# Purpose: ensure operations docs reference make targets instead of raw script invocations.
# Inputs: docs/operations/**/*.md and `make help` target list.
# Outputs: non-zero exit on docs without any make target references.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OPS_DOCS = ROOT / "docs" / "operations"

help_out = subprocess.check_output(["make", "help"], cwd=ROOT, text=True)
targets: list[str] = []
for line in help_out.splitlines():
    if not line.startswith("  "):
        continue
    targets.extend(line.strip().split())

ops_targets = [t for t in sorted(set(targets)) if t.startswith("ops-") or t.startswith("e2e-") or t == "observability-check"]
pattern = re.compile(r"`(" + "|".join(re.escape(t) for t in ops_targets) + r")`")

errors: list[str] = []
for md in sorted(OPS_DOCS.rglob("*.md")):
    text = md.read_text(encoding="utf-8", errors="ignore")
    if pattern.search(text):
        continue
    errors.append(f"{md.relative_to(ROOT)}: missing ops make target reference")

if errors:
    print("ops docs make-target contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops docs make-target contract passed")
