# Bijux Error Code Registry

Stable machine error codes:

- `usage_error`
- `validation_error`
- `dependency_failure`
- `plugin_missing`
- `plugin_incompatible`
- `plugin_metadata_error`
- `plugin_exec_failed`
- `plugin_failed`
- `internal_error`
- `rate_limited`
- `timeout`

Rules:

- Codes are append-only; do not repurpose existing meanings.
- Any new code requires docs update and contract tests.
