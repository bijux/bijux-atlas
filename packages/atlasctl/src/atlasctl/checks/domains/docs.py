from __future__ import annotations

from ..model import CheckDef
from ..tools.docs_domain import CHECKS as DOCS_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(DOCS_CHECKS)

__all__ = ["CHECKS"]
