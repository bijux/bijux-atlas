from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from .runtime import run_load_action


def run_load_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_load_action(ctx, str(getattr(ns, 'ops_load_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


__all__ = ['run_load_command']
