from __future__ import annotations

from ..repo.native import check_atlas_scripts_cli_contract, check_layout_contract
from ..base import CheckCategory, CheckDef
from .schema_contracts import (
    check_schema_catalog_files_exist,
    check_schema_catalog_integrity,
    check_schema_catalog_sorted,
    check_schema_catalog_referenced,
    check_schema_disk_files_listed,
    check_schema_id_naming,
    check_schema_goldens_validate,
    check_schema_catalog_ssot,
    check_schema_samples_validate,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("contracts.atlasctl_cli", "contracts", "validate atlasctl CLI contract surface", 1200, check_atlas_scripts_cli_contract, category=CheckCategory.CONTRACT, fix_hint="Update CLI contract docs/tests to match behavior."),
    CheckDef("contracts.layout", "contracts", "validate repository layout contract", 1200, check_layout_contract, category=CheckCategory.CONTRACT, fix_hint="Fix layout violations reported by the contract checker.", slow=True, external_tools=("bash",)),
    CheckDef("contracts.schema_catalog", "contracts", "validate schema catalog for duplicate names and missing files", 1200, check_schema_catalog_integrity, category=CheckCategory.CONTRACT, fix_hint="Fix catalog duplicates and missing schema files."),
    CheckDef("contracts.schema_catalog_files_exist", "contracts", "ensure schemas listed in catalog exist on disk", 1200, check_schema_catalog_files_exist, category=CheckCategory.CONTRACT, fix_hint="Add missing schema files or fix catalog file entries."),
    CheckDef("contracts.schema_disk_files_listed", "contracts", "ensure no schema file exists outside catalog", 1200, check_schema_disk_files_listed, category=CheckCategory.CONTRACT, fix_hint="Add schema file to catalog.json or remove stale schema file."),
    CheckDef("contracts.schema_catalog_sorted", "contracts", "ensure schema catalog order is deterministic", 1200, check_schema_catalog_sorted, category=CheckCategory.CONTRACT, fix_hint="Sort catalog.json entries by schema name."),
    CheckDef("contracts.schema_id_naming", "contracts", "enforce schema id naming/version suffix policy", 1200, check_schema_id_naming, category=CheckCategory.CONTRACT, fix_hint="Use atlasctl.<name>.v<major> and keep catalog version aligned with suffix."),
    CheckDef("contracts.schema_catalog_referenced", "contracts", "ensure schema catalog contains only referenced schemas", 1200, check_schema_catalog_referenced, category=CheckCategory.CONTRACT, fix_hint="Remove or reference orphan schemas in catalog."),
    CheckDef("contracts.schema_catalog_ssot", "contracts", "enforce contracts catalog.json as schema SSOT", 1200, check_schema_catalog_ssot, category=CheckCategory.CONTRACT, fix_hint="Route schema catalog access through atlasctl.contracts.catalog only."),
    CheckDef("contracts.schema_samples", "contracts", "validate sample payloads against declared schemas", 1200, check_schema_samples_validate, category=CheckCategory.CONTRACT, fix_hint="Update sample payloads or schema definitions."),
    CheckDef("contracts.schema_goldens", "contracts", "validate JSON goldens against declared schemas", 1200, check_schema_goldens_validate, category=CheckCategory.CONTRACT, fix_hint="Fix golden payload shape/schema alignment."),
)
