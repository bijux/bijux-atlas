from __future__ import annotations

from ..model import CheckDef
from ..tools.ops import CHECKS as OPS_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(OPS_CHECKS)


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
