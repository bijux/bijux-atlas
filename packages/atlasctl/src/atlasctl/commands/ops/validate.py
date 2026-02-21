"""Ops validation wrapper."""

from __future__ import annotations

from .runtime import run_ops_command


def run_validate(ctx, ns):
    return run_ops_command(ctx, ns)
