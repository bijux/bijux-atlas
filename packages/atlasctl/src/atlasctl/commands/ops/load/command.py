from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext
from atlasctl.commands._shared_runtime import run_group_action

from .runtime import run_load_action


def run_load_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_load_action(ctx, str(getattr(ns, 'ops_load_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


def configure_load_command(*_args: object, **_kwargs: object) -> None:
    """CLI intent marker; parser wiring lives in ops root parser."""
    return None


__all__ = ['run_load_command']
