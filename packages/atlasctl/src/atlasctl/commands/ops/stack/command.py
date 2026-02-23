from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext
from atlasctl.commands._shared_runtime import run_group_action

from .runtime import run_stack_action


def configure_ops_stack_parser_intent() -> None:
    """Intent marker for repo CLI module checks."""
    return None


def run_stack_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    action = str(getattr(ns, "ops_stack_cmd", "") or "").strip()
    return run_stack_action(ctx, action, getattr(ns, "report", "text"))


__all__ = ["run_stack_command"]
