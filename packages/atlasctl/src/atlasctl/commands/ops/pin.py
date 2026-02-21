"""Ops pin/update wrapper."""

from __future__ import annotations

from .runtime import run_ops_command


def run_pin(ctx, ns):
    return run_ops_command(ctx, ns)
