from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from .runtime import run_stack_action


def run_stack_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    action = str(getattr(ns, "ops_stack_cmd", "") or "").strip()
    return run_stack_action(ctx, action, getattr(ns, "report", "text"))


__all__ = ["run_stack_command"]
