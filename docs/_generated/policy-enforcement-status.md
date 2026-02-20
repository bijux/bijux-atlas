# Policy Enforcement Status

- Owner: `atlas-platform`
- Generated from: `configs/policy/policy-enforcement-coverage.json`
- Hard policy coverage: `4/4` (`100%`)

| Policy | Class | Pass Test | Fail Test | Status |
| --- | --- | --- | --- | --- |
| `dataset_identity_required` | `hard` | `ApiErrorCode::MissingDatasetDimension` | `MissingDatasetDimension` | `PASS` |
| `no_network_unit_tests` | `hard` | `scripts/areas/public/no-network-unit-tests.sh` | `network usage in unit tests is forbidden` | `PASS` |
| `policy_relaxations_registry` | `hard` | `scripts/areas/public/policy-audit.py` | `exception {entry['id']} expired on` | `PASS` |
| `query_budget_rejections` | `hard` | `expensive_include_is_policy_gated_by_projection_limits` | `memory_pressure_guards_reject_large_response_without_cascading_failure` | `PASS` |
