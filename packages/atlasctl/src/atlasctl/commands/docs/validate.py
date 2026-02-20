"""Docs validation facade."""

from __future__ import annotations

from .legacy import run_docs_command


def run_validate(ctx, ns):
    return run_docs_command(ctx, ns)
