# Policy Authoring Guide

## Add A Policy
1. Add a stable policy id to `POLICY_REGISTRY` in `src/schema.rs`.
2. Choose one category: `repo`, `docs`, `ops`, or `configs`.
3. Add policy data fields to the document schema struct if needed.
4. Add evaluation logic in `evaluate_policy_set_pure`.
5. Keep evaluation pure: inputs must come from `PolicyInputSnapshot`.
6. Add table tests to `tests/policy_evaluation_table.rs`.
7. Update `ops/inventory/policies/dev-atlas-policy.schema.json` when contract changes.
8. Update `ops/inventory/policies/dev-atlas-policy.json` with default values.
9. Run schema validation tests.
10. Run full crate tests.

## Add A Relaxation
1. Add a `relaxations` row with `policy_id`, `reason`, and `expires_on` (`YYYY-MM-DD`).
2. Keep `policy_id` in registry.
3. Add or update expiry enforcement tests in `tests/relaxation_expiry.rs`.
4. Remove expired relaxations proactively.
