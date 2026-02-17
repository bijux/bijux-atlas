# PUBLIC API: bijux-atlas-policies

Stable exports:
- `CRATE_NAME`
- `PolicyConfig`, `PolicySchema`
- `QueryBudget`, `CacheBudget`, `RateLimitPolicy`, `ConcurrencyBulkheads`, `TelemetryPolicy`
- `PolicyValidationError`
- `load_policy_from_workspace`, `policy_config_path`, `policy_schema_path`
- `validate_policy_config`, `validate_schema_version_transition`
- `canonical_config_json`
- `MAX_LOC_HARD`, `MAX_DEPTH_HARD`, `MAX_RS_FILES_PER_DIR_HARD`, `MAX_MODULES_PER_DIR_HARD`
- `MIN_POLICY_SCHEMA_VERSION`, `MAX_SCHEMA_BUMP_STEP`

Only listed exports are allowed from `src/lib.rs` unless this file is updated.
