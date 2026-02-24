# PUBLIC API: bijux-atlas-policies

Stability reference: [Stability Levels](../../../docs/_style/stability-levels.md)

Stable exports:
- `CRATE_NAME`
- `PolicyConfig`, `PolicySchema`
- `QueryBudget`, `CacheBudget`, `RateLimitPolicy`, `ConcurrencyBulkheads`, `TelemetryPolicy`
- `PolicyValidationError`
- `load_policy_from_workspace`, `policy_config_path`, `policy_schema_path`
- `validate_policy_config`, `validate_schema_version_transition`
- `canonical_config_json`
- `MIN_POLICY_SCHEMA_VERSION`, `MAX_SCHEMA_BUMP_STEP`

Only listed exports are allowed from `src/lib.rs` unless this file is updated.
