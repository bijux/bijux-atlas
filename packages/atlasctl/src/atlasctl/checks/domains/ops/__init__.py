from __future__ import annotations

from ...core.base import CheckDef
from .docker import CHECKS as DOCKER_CHECKS
from .contracts import CHECKS as OPS_CHECKS

CHECKS: tuple[CheckDef, ...] = tuple((*OPS_CHECKS, *DOCKER_CHECKS))

__all__ = ["CHECKS"]
