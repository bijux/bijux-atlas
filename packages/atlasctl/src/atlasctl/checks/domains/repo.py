from __future__ import annotations

from ..model import CheckDef
from ..repo import CHECKS as REPO_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(REPO_CHECKS)

__all__ = ["CHECKS"]
