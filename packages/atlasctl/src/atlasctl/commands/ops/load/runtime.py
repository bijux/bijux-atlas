from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_load_action(ctx: RunContext, action: str, report: str) -> int:
    ns = argparse.Namespace(ops_cmd="load", ops_load_cmd=action, report=report)
    if action == "run":
        ns.suite = "mixed-80-20"
        ns.out = "artifacts/perf/results"
    if action == "compare":
        ns.baseline = ""
        ns.current = ""
        ns.out = "artifacts/reports/atlasctl/ops-load-compare.json"
    return _run_ops(ctx, ns)


__all__ = ['run_load_action']
