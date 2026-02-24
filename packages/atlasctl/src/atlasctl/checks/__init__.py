"""Atlasctl check system public API.

SSOT rules:
- Check types and interfaces live in ``atlasctl.checks.model``.
- Registry selection comes from ``atlasctl.checks.registry``.
- Execution is performed by engine runtime; command modules remain thin wrappers.
"""

from .model import (
    CheckCategory,
    CheckContext,
    CheckDef,
    CheckFn,
    CheckId,
    CheckOutcome,
    CheckResult,
    CheckRunReport,
    CheckSelector,
    CheckStatus,
    DomainId,
    Effect,
    OwnerId,
    ResultCode,
    Severity,
    Tag,
    Violation,
)
from .runner import extract_failures, run_checks, top_n_slowest
from .violations import v
from .registry import check_tags, get_check, list_checks, list_domains


__all__ = [
    "CheckCategory",
    "CheckContext",
    "CheckDef",
    "CheckFn",
    "CheckId",
    "CheckOutcome",
    "CheckResult",
    "CheckRunReport",
    "CheckSelector",
    "CheckStatus",
    "DomainId",
    "Effect",
    "OwnerId",
    "ResultCode",
    "Severity",
    "Tag",
    "Violation",
    "check_tags",
    "get_check",
    "list_checks",
    "list_domains",
    "run_checks",
    "extract_failures",
    "top_n_slowest",
    "v",
]
