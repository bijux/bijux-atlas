from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext
from atlasctl.commands._shared_runtime import run_group_action

from .runtime import run_e2e_action


def run_e2e_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_e2e_action(ctx, str(getattr(ns, 'ops_e2e_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


__all__ = ['run_e2e_command']
