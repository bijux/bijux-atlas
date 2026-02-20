#!/usr/bin/env python3
# Purpose: enforce canonical single entrypoints for deploy/publish/smoke/obs verify flows.
# Inputs: makefiles/ops.mk and selected scripts.
# Outputs: non-zero if non-canonical entrypoints are used.
from __future__ import annotations
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
ops_mk = (ROOT / "makefiles/ops.mk").read_text(encoding="utf-8")
errors: list[str] = []

if "./ops/run/deploy-atlas.sh" not in ops_mk:
    errors.append("ops-deploy must call ./ops/run/deploy-atlas.sh")
if "./ops/run/undeploy.sh" not in ops_mk:
    errors.append("ops-undeploy must call ./ops/run/undeploy.sh")
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
if "ops-stack-up-legacy" in ops_mk or "ops-stack-down-legacy" in ops_mk:
    errors.append("legacy stack entrypoint targets must not exist in makefiles/ops.mk")
if "ops-check-legacy" in ops_mk or "ops-smoke-legacy" in ops_mk:
    errors.append("legacy ops-check/ops-smoke entrypoint targets must not exist in makefiles/ops.mk")

stack_up_block = ops_mk.split("ops-stack-up:", 1)[1].split("\n\n", 1)[0] if "ops-stack-up:" in ops_mk else ""
stack_down_block = ops_mk.split("ops-stack-down:", 1)[1].split("\n\n", 1)[0] if "ops-stack-down:" in ops_mk else ""
if "./ops/run/stack-up.sh" not in stack_up_block:
    errors.append("ops-stack-up must call ./ops/run/stack-up.sh")
if "./ops/run/stack-down.sh" not in stack_down_block:
    errors.append("ops-stack-down must call ./ops/run/stack-down.sh")
if "ops-stack-uninstall:" in ops_mk:
    errors.append("ops-stack-uninstall duplicate path is forbidden; use ops-stack-down")
if "./ops/e2e/scripts/up.sh" in ops_mk or "./ops/e2e/scripts/down.sh" in ops_mk:
    errors.append("ops-stack up/down must not call legacy e2e up/down scripts")

for path in [ROOT / "ops/e2e/realdata/run_single_release.sh", ROOT / "ops/e2e/realdata/run_two_release_diff.sh"]:
    txt = path.read_text(encoding="utf-8")
    if "/ops/e2e/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/run/deploy-atlas.sh")
    if "/ops/k8s/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/run/deploy-atlas.sh")
    if "/ops/e2e/runner/publish_dataset.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use ops/datasets/scripts/publish_by_name.sh")

if errors:
    print("canonical entrypoint check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("canonical entrypoint check passed")
