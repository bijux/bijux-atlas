from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_obs_action(ctx: RunContext, action: str, report: str) -> int:
    ns = argparse.Namespace(ops_cmd="obs", ops_obs_cmd=action, report=report)
    if action == "verify":
        ns.suite = "full"
        ns.args = []
    if action == "report":
        ns.out = "artifacts/reports/atlasctl/ops-obs-report.json"
    if action == "drill":
        ns.drill = ""
    return _run_ops(ctx, ns)


__all__ = ['run_obs_action']
