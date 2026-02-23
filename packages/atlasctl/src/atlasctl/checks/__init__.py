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
from .runner import run_checks
from .registry import check_tags, get_check, list_checks, list_domains


def domains() -> list[str]:
    from ..engine.runner import domains as _domains

    return _domains()


def run_domain(*args: object, **kwargs: object):  # noqa: ANN002, ANN003
    from ..engine.runner import run_domain as _run_domain

    return _run_domain(*args, **kwargs)

__all__ = [
    "CheckCategory",
    "CheckContext",
    "CheckDef",
    "CheckFn",
    "CheckId",
    "CheckOutcome",
    "CheckResult",
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
    "domains",
    "get_check",
    "list_checks",
    "list_domains",
    "run_domain",
    "run_checks",
]
