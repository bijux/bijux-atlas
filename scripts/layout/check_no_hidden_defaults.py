#!/usr/bin/env python3
# Purpose: enforce no hidden defaults in ops run entrypoints.
# Inputs: ops/run/*.sh.
# Outputs: non-zero if scripts skip ops_env_load or hide env defaults.
from __future__ import annotations
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
errors: list[str] = []
allowed = {("clean.sh", "OPS_RETENTION_DAYS"), ("obs-up.sh", "ATLAS_OBS_PROFILE"), ("report.sh", "OPS_RUN_DIR")}
for p in sorted((ROOT / "ops/run").glob("*.sh")):
    t = p.read_text(encoding="utf-8")
    if "ops_env_load" not in t:
        errors.append(f"{p.relative_to(ROOT)}: missing ops_env_load")
    for m in re.finditer(r"\$\{(ATLAS_[A-Z0-9_]+|OPS_[A-Z0-9_]+):-", t):
        if (p.name, m.group(1)) in allowed:
            continue
        errors.append(f"{p.relative_to(ROOT)}: hidden default for {m.group(1)} not allowed in run wrapper")

if errors:
    print("no-hidden-defaults check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("no-hidden-defaults check passed")
