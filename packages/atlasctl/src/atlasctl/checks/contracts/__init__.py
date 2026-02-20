from __future__ import annotations

from ..repo.legacy_native import check_atlas_scripts_cli_contract, check_layout_contract
from ..base import CheckCategory, CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("contracts.atlasctl_cli", "contracts", "validate atlasctl CLI contract surface", 1200, check_atlas_scripts_cli_contract, category=CheckCategory.CONTRACT, fix_hint="Update CLI contract docs/tests to match behavior."),
    CheckDef("contracts.layout", "contracts", "validate repository layout contract", 1200, check_layout_contract, category=CheckCategory.CONTRACT, fix_hint="Fix layout violations reported by the contract checker.", slow=True, external_tools=("bash",)),
)
