# Policy Evolution

Versioning:
- Schema changes must be forward-only.
- Policy content changes require schema version bump if they alter compatibility assumptions.

Compatibility:
- Allowed transition step is controlled by `MAX_SCHEMA_BUMP_STEP`.
- Compatibility matrix is validated in tests.

Change checklist:
1. Update `configs/policy/policy.schema.json`.
2. Update `configs/policy/policy.json`.
3. Update `PolicySchemaVersion` if version changes.
4. Update tests for new/changed knobs.
