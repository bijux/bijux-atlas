from __future__ import annotations

from ..model import CheckDef
from ..tools.ops_domain import CHECKS as OPS_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple(OPS_CHECKS)

__all__ = ["CHECKS"]
