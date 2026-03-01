# PUBLIC API: bijux-atlas-policies

Stability reference: [Stability Levels](../../../docs/_internal/governance/style/stability-levels.md)

Stable exports:
- `CRATE_NAME`
- `PolicySet`, `PolicyConfig`, `PolicySchema`
- `PolicyViolation`, `PolicySeverity`, `RepositoryMetrics`
- `parse_policy_set_json`, `validate_policy_set`
- `evaluate_policy_set`, `evaluate_repository_metrics`
- `load_policy_set_from_workspace`, `policy_config_path`, `policy_schema_path`
- `PolicyValidationError`
- `validate_policy_config`, `validate_schema_version_transition`
- `canonical_config_json`
- `MIN_POLICY_SCHEMA_VERSION`, `MAX_SCHEMA_BUMP_STEP`

Only listed exports are allowed from `src/lib.rs` unless this file is updated.
