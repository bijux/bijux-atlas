#!/usr/bin/env python3
# Purpose: ensure operations docs reference make targets instead of raw script invocations.
# Inputs: docs/operations/**/*.md and `make help` target list.
# Outputs: non-zero exit on missing area-level make target references or direct script paths.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OPS_DOCS = ROOT / "docs" / "operations"

help_out = subprocess.check_output(["make", "help"], cwd=ROOT, text=True)
targets: list[str] = []
for line in help_out.splitlines():
    if not line.startswith("  "):
        continue
    targets.extend(line.strip().split())

ops_targets = [t for t in sorted(set(targets)) if t.startswith("ops-") or t.startswith("e2e-") or t == "observability-check"]
target_pattern = re.compile(r"`(" + "|".join(re.escape(t) for t in ops_targets) + r")`")
make_cmd_pattern = re.compile(r"\bmake\s+(" + "|".join(re.escape(t) for t in ops_targets) + r")\b")

errors: list[str] = []
area_has_target: dict[Path, bool] = {}
area_index: dict[Path, Path] = {}

for md in sorted(OPS_DOCS.rglob("*.md")):
    text = md.read_text(encoding="utf-8", errors="ignore")
    area = md.parent
    area_has_target.setdefault(area, False)
    if target_pattern.search(text) or make_cmd_pattern.search(text):
        area_has_target[area] = True
    if md.name == "INDEX.md":
        area_index[area] = md
    if re.search(r"(^|\\s)\\./(ops|scripts)/", text):
        errors.append(f"{md.relative_to(ROOT)}: direct script path reference found; use make target")

for area, has_target in sorted(area_has_target.items()):
    if has_target:
        continue
    index = area_index.get(area)
    if index is not None:
        errors.append(f"{index.relative_to(ROOT)}: missing ops make target reference for area")
    else:
        errors.append(f"{area.relative_to(ROOT)}: missing INDEX.md with ops make target reference")

if errors:
    print("ops docs make-target contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops docs make-target contract passed")
