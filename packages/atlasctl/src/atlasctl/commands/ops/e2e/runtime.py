from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_e2e_action(ctx: RunContext, action: str, report: str) -> int:
    ns = argparse.Namespace(ops_cmd="e2e", ops_e2e_cmd=action, report=report)
    if action == "run":
        ns.suite = "smoke"
        ns.scenario = None
    if action == "validate-results":
        ns.in_file = "artifacts/reports/atlasctl/ops-e2e-results.json"
    return _run_ops(ctx, ns)


__all__ = ['run_e2e_action']
