"""Canonical checks command entrypoint."""

from ..commands.check.command import configure_check_parser, configure_checks_parser, run_check_command

__all__ = ["configure_check_parser", "configure_checks_parser", "run_check_command"]
