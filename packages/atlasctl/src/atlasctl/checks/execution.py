"""Compatibility shim for `atlasctl.checks.execution`."""

from .core.execution import CommandCheckDef, run_command_checks, run_function_checks

__all__ = ["CommandCheckDef", "run_command_checks", "run_function_checks"]
