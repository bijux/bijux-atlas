#!/usr/bin/env python3
# Purpose: enforce canonical single entrypoints for deploy/publish/smoke/obs verify flows.
# Inputs: makefiles/ops.mk and selected scripts.
# Outputs: non-zero if non-canonical entrypoints are used.
from __future__ import annotations
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
ops_mk = (ROOT / "makefiles/ops.mk").read_text(encoding="utf-8")
errors: list[str] = []

if "./ops/run/deploy-atlas.sh" not in ops_mk:
    errors.append("ops-deploy must call ./ops/run/deploy-atlas.sh")
if "./ops/k8s/scripts/deploy_atlas.sh" in ops_mk:
    errors.append("ops-deploy must not call ./ops/k8s/scripts/deploy_atlas.sh directly")
if "./ops/e2e/scripts/deploy_atlas.sh" in ops_mk:
    errors.append("ops-deploy must not call ./ops/e2e/scripts/deploy_atlas.sh directly")
if "./ops/datasets/scripts/publish_by_name.sh" not in ops_mk:
    errors.append("ops-publish must call ./ops/datasets/scripts/publish_by_name.sh")
if "./ops/e2e/scripts/smoke_queries.sh" not in ops_mk:
    errors.append("ops-smoke must call ./ops/e2e/scripts/smoke_queries.sh")
if "./ops/run/obs-verify.sh" not in ops_mk and "./ops/obs/scripts/verify_pack.sh" not in ops_mk:
    errors.append("ops observability verify flow must use verify_pack entrypoint")

for path in [ROOT / "ops/e2e/realdata/run_single_release.sh", ROOT / "ops/e2e/realdata/run_two_release_diff.sh"]:
    txt = path.read_text(encoding="utf-8")
    if "/ops/e2e/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/run/deploy-atlas.sh")
    if "/ops/k8s/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/run/deploy-atlas.sh")
    if "/ops/e2e/scripts/publish_dataset.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/datasets/scripts/publish_by_name.sh")

if errors:
    print("canonical entrypoint check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("canonical entrypoint check passed")
