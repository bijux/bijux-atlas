from __future__ import annotations

from ..model import CheckCategory, CheckDef
from ..tools.repo_domain.native import check_python_lock, check_python_migration_exceptions_expiry

CHECKS: tuple[CheckDef, ...] = (
    CheckDef(
        "python.lock",
        "python",
        "validate python lock format",
        800,
        check_python_lock,
        category=CheckCategory.CONTRACT,
        fix_hint="Regenerate and normalize python lock entries.",
    ),
    CheckDef(
        "python.migration_exceptions_expiry",
        "python",
        "fail on expired python migration exceptions",
        800,
        check_python_migration_exceptions_expiry,
        category=CheckCategory.POLICY,
        fix_hint="Update or remove expired migration exceptions.",
    ),
)


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
