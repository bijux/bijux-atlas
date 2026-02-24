from __future__ import annotations

from ..model import CheckDef
from ..tools.repo_domain import CHECKS as REPO_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(REPO_CHECKS)

__all__ = ["CHECKS"]
