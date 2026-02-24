from __future__ import annotations

from ..model import CheckDef
from .repo import CHECKS as REPO_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(check for check in REPO_CHECKS if str(check.domain) == "python")

__all__ = ["CHECKS"]
