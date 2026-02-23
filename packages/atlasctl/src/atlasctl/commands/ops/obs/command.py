from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext
from atlasctl.commands._shared_runtime import run_group_action

from .runtime import run_obs_action


def run_obs_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_obs_action(ctx, str(getattr(ns, 'ops_obs_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


__all__ = ['run_obs_command']
