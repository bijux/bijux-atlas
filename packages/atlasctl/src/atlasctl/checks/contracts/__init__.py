from __future__ import annotations

from ..repo.legacy_native import check_atlas_scripts_cli_contract, check_layout_contract
from ..base import CheckCategory, CheckDef
from .schema_contracts import check_schema_catalog_integrity, check_schema_samples_validate

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("contracts.atlasctl_cli", "contracts", "validate atlasctl CLI contract surface", 1200, check_atlas_scripts_cli_contract, category=CheckCategory.CONTRACT, fix_hint="Update CLI contract docs/tests to match behavior."),
    CheckDef("contracts.layout", "contracts", "validate repository layout contract", 1200, check_layout_contract, category=CheckCategory.CONTRACT, fix_hint="Fix layout violations reported by the contract checker.", slow=True, external_tools=("bash",)),
    CheckDef("contracts.schema_catalog", "contracts", "validate schema catalog for duplicate names and missing files", 1200, check_schema_catalog_integrity, category=CheckCategory.CONTRACT, fix_hint="Fix catalog duplicates and missing schema files."),
    CheckDef("contracts.schema_samples", "contracts", "validate sample payloads against declared schemas", 1200, check_schema_samples_validate, category=CheckCategory.CONTRACT, fix_hint="Update sample payloads or schema definitions."),
)
