#!/usr/bin/env python3
# Purpose: enforce no hidden defaults in ops run entrypoints.
# Inputs: ops/run/*.sh.
# Outputs: non-zero if scripts skip ops_env_load or hide env defaults.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
errors: list[str] = []
allowed = {
    ("clean.sh", "OPS_RETENTION_DAYS"),
    ("obs-up.sh", "ATLAS_OBS_PROFILE"),
    ("report.sh", "OPS_RUN_DIR"),
    ("root-lanes.sh", "OPS_RUN_ID"),
    ("root-lanes.sh", "OPS_RUN_DIR"),
    ("stack-up.sh", "ATLAS_KIND_REGISTRY_ENABLE"),
}
extra_allowed = {
    ("ops/k8s/scripts/clean_uninstall.sh", "ATLAS_E2E_NAMESPACE"),
    ("ops/k8s/scripts/clean_uninstall.sh", "ATLAS_NS"),
    ("ops/k8s/scripts/clean_uninstall.sh", "ATLAS_E2E_RELEASE_NAME"),
}
for p in sorted((ROOT / "ops/run").glob("*.sh")):
    t = p.read_text(encoding="utf-8")
    if "ops_env_load" not in t:
        errors.append(f"{p.relative_to(ROOT)}: missing ops_env_load")
    for m in re.finditer(r"\$\{(ATLAS_[A-Z0-9_]+|OPS_[A-Z0-9_]+):-", t):
        if (p.name, m.group(1)) in allowed:
            continue
        errors.append(f"{p.relative_to(ROOT)}: hidden default for {m.group(1)} not allowed in run wrapper")

# Tightened coverage: k8s scripts may not hide ATLAS/OPS/PROFILE defaults without explicit allow.
for p in sorted((ROOT / "ops/k8s/scripts").rglob("*.sh")):
    t = p.read_text(encoding="utf-8")
    rel = p.relative_to(ROOT).as_posix()
    for m in re.finditer(r"\$\{(ATLAS_[A-Z0-9_]+|OPS_[A-Z0-9_]+|PROFILE):-", t):
        if (rel, m.group(1)) in extra_allowed:
            continue
        if "ops_layer_" in t or "ops_layer_contract_get" in t:
            continue
        errors.append(f"{rel}: hidden default for {m.group(1)} not allowed")

# Tightened coverage: k8s values should not hardcode namespace.
for p in sorted((ROOT / "ops/k8s/values").glob("*.y*ml")):
    text = p.read_text(encoding="utf-8")
    if re.search(r"^\s*namespace:\s*[a-z0-9-]+\s*$", text, flags=re.M):
        errors.append(f"{p.relative_to(ROOT)}: namespace must come from contract/env, not hardcoded in values")

if errors:
    print("no-hidden-defaults check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("no-hidden-defaults check passed")
