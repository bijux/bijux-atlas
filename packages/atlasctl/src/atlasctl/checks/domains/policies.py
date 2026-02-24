from __future__ import annotations

from ..model import CheckDef
from ..tools.policies_domain import CHECKS as POLICIES_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(POLICIES_CHECKS)


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
