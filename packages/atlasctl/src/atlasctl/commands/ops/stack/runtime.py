from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from ..runtime_modules.ops_runtime_run import run_ops_command as _run_ops


def run_stack_action(ctx: RunContext, action: str, report: str) -> int:
    return _run_ops(ctx, argparse.Namespace(ops_cmd="stack", ops_stack_cmd=action, report=report))


__all__ = ["run_stack_action"]
