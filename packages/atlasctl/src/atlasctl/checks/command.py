"""Canonical checks command entrypoint."""

from ..commands.check.legacy import configure_check_parser, run_check_command

__all__ = ["configure_check_parser", "run_check_command"]
