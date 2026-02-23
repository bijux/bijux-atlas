"""LEGACY facade for check execution runtime.

Cutoff: 2026-04-01. Canonical home is `atlasctl.checks.engine.execution`.
"""
from __future__ import annotations

from ..engine.execution import CommandCheckDef, run_command_checks, run_function_checks

__all__ = ["CommandCheckDef", "run_command_checks", "run_function_checks"]
