"""Compatibility shim for `atlasctl.checks.base`."""

from .core.base import Check, CheckCategory, CheckDef, CheckFunc, CheckResult, Severity

__all__ = [
    "Check",
    "CheckCategory",
    "CheckDef",
    "CheckFunc",
    "CheckResult",
    "Severity",
]
