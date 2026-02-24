from __future__ import annotations

from ..model import CheckDef
from ..tools.repo_domain import CHECKS as REPO_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(check for check in REPO_CHECKS if str(check.domain) == "python")

__all__ = ["CHECKS"]
