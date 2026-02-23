from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_datasets_action(ctx: RunContext, action: str, report: str) -> int:
    ns = argparse.Namespace(ops_cmd="datasets", ops_datasets_cmd=action, report=report)
    if action == "qc":
        ns.ops_datasets_qc_cmd = "summary"
        ns.args = []
    return _run_ops(ctx, ns)


__all__ = ['run_datasets_action']
