from __future__ import annotations

from ...core.base import CheckDef
from ..dev.python import CHECKS as PYTHON_CHECKS
from .contracts import CHECKS as CONTRACTS_CHECKS
from .licensing import CHECKS as LICENSING_CHECKS
from .make import CHECKS as MAKE_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple((*LICENSING_CHECKS, *MAKE_CHECKS, *CONTRACTS_CHECKS, *PYTHON_CHECKS))

__all__ = ["CHECKS"]
