"""Docs generation facade."""

from __future__ import annotations

from .runtime import run_docs_command


def run_generate(ctx, ns):
    return run_docs_command(ctx, ns)
