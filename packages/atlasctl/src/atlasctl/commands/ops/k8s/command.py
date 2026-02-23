from __future__ import annotations

import argparse

from atlasctl.core.context import RunContext

from .runtime import run_k8s_action


def run_k8s_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_k8s_action(ctx, str(getattr(ns, 'ops_k8s_cmd', '') or '').strip(), getattr(ns, 'report', 'text'))


__all__ = ['run_k8s_command']
