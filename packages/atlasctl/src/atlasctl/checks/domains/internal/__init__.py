from __future__ import annotations

from ...core.base import CheckDef
from .checks import CHECKS as INTERNAL_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(INTERNAL_CHECKS)

__all__ = ["CHECKS"]
