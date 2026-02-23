from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_k8s_action(ctx: RunContext, action: str, report: str) -> int:
    ns = argparse.Namespace(ops_cmd="k8s", ops_k8s_cmd=action, report=report)
    if action == "render":
        ns.env = "kind"
        ns.out = "artifacts/reports/atlasctl/ops-k8s-render.json"
    if action == "validate":
        ns.in_file = "artifacts/reports/atlasctl/ops-k8s-render.json"
    if action == "validate-configmap-keys":
        ns.namespace = None
        ns.service_name = None
    if action == "diff":
        ns.in_file = "artifacts/reports/atlasctl/ops-k8s-render.json"
        ns.golden = "ops/k8s/tests/goldens/render-kind.summary.json"
    return _run_ops(ctx, ns)


__all__ = ['run_k8s_action']
