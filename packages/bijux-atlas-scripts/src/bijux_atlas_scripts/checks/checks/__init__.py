from __future__ import annotations

from ...check.native import check_script_help, check_script_ownership
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("checks/help-coverage", "checks", 1500, check_script_help),
    CheckDef("checks/ownership-coverage", "checks", 1500, check_script_ownership),
)
