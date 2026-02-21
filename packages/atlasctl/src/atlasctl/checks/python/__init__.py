from __future__ import annotations

from ..repo.native import check_python_lock, check_python_migration_exceptions_expiry
from ..framework import CheckCategory, CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("python.lock", "python", "validate python lock format", 800, check_python_lock, category=CheckCategory.CONTRACT, fix_hint="Regenerate and normalize python lock entries."),
    CheckDef("python.migration_exceptions_expiry", "python", "fail on expired python migration exceptions", 800, check_python_migration_exceptions_expiry, category=CheckCategory.POLICY, fix_hint="Update or remove expired migration exceptions."),
)
