from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from .runtime import run_datasets_action


def run_datasets_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_datasets_action(ctx, str(getattr(ns, 'ops_datasets_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


__all__ = ['run_datasets_command']
