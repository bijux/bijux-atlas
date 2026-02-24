from __future__ import annotations

from ..model import CheckDef
from ..tools.policies_domain.make import CHECKS as POLICIES_MAKE_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(check for check in POLICIES_MAKE_CHECKS if str(check.domain) == "make")


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
