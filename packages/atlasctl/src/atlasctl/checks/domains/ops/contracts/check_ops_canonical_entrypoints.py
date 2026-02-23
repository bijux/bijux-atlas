#!/usr/bin/env python3
# Purpose: enforce canonical single entrypoints for deploy/publish/smoke/obs verify flows.
# Inputs: makefiles/ops.mk and selected scripts.
# Outputs: non-zero if non-canonical entrypoints are used.
from __future__ import annotations

import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
ops_mk = (ROOT / "makefiles/ops.mk").read_text(encoding="utf-8")
errors: list[str] = []

if "./bin/atlasctl ops deploy --report text apply" not in ops_mk:
    errors.append("ops-deploy must call ./bin/atlasctl ops deploy --report text apply")
if "./bin/atlasctl ops deploy --report text rollback" not in ops_mk:
    errors.append("ops-undeploy must call ./bin/atlasctl ops deploy --report text rollback")
if "./ops/k8s/scripts/deploy_atlas.sh" in ops_mk:
    errors.append("ops-deploy must not call ./ops/k8s/scripts/deploy_atlas.sh directly")
if "./ops/e2e/scripts/deploy_atlas.sh" in ops_mk:
    errors.append("ops-deploy must not call ./ops/e2e/scripts/deploy_atlas.sh directly")
if "./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/datasets/publish_by_name.py" not in ops_mk:
    errors.append("ops-publish must call atlasctl-owned datasets publish_by_name.py entrypoint")
if "./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/smoke_queries.py" not in ops_mk:
    errors.append("ops-smoke must call atlasctl-owned e2e runtime smoke_queries.py entrypoint")
if "./bin/atlasctl ops obs --report text verify" not in ops_mk and "./ops/obs/scripts/verify_pack.sh" not in ops_mk:
    errors.append("ops observability verify flow must use atlasctl ops obs verify (or verify_pack entrypoint)")
if "ops-stack-up-legacy" in ops_mk or "ops-stack-down-legacy" in ops_mk:
    errors.append("legacy stack entrypoint targets must not exist in makefiles/ops.mk")
if "ops-check-legacy" in ops_mk or "ops-smoke-legacy" in ops_mk:
    errors.append("legacy ops-check/ops-smoke entrypoint targets must not exist in makefiles/ops.mk")

stack_up_block = ops_mk.split("ops-stack-up:", 1)[1].split("\n\n", 1)[0] if "ops-stack-up:" in ops_mk else ""
stack_down_block = ops_mk.split("ops-stack-down:", 1)[1].split("\n\n", 1)[0] if "ops-stack-down:" in ops_mk else ""
if "./bin/atlasctl ops stack --report text up" not in stack_up_block:
    errors.append("ops-stack-up must call ./bin/atlasctl ops stack --report text up")
if "./bin/atlasctl ops stack --report text down" not in stack_down_block:
    errors.append("ops-stack-down must call ./bin/atlasctl ops stack --report text down")
if "ops-stack-uninstall:" in ops_mk:
    errors.append("ops-stack-uninstall duplicate path is forbidden; use ops-stack-down")
if "./ops/e2e/scripts/up.sh" in ops_mk or "./ops/e2e/scripts/down.sh" in ops_mk:
    errors.append("ops-stack up/down must not call legacy e2e up/down scripts")

for path in [ROOT / "ops/e2e/realdata/run_single_release.sh", ROOT / "ops/e2e/realdata/run_two_release_diff.sh"]:
    if not path.exists():
        continue
    txt = path.read_text(encoding="utf-8")
    if "/ops/e2e/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use atlasctl ops deploy")
    if "/ops/k8s/scripts/deploy_atlas.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must use atlasctl ops deploy")
    if "/ops/e2e/runner/publish_dataset.sh" in txt:
        errors.append(f"{path.relative_to(ROOT)} must not call legacy e2e publish_dataset.sh directly")

if errors:
    print("canonical entrypoint check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("canonical entrypoint check passed")
