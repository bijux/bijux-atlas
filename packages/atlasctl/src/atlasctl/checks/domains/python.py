from __future__ import annotations

from ..model import CheckDef
from ..tools.repo_domain import CHECKS as REPO_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(check for check in REPO_CHECKS if str(check.domain) == "python")


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
