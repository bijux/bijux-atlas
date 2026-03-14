# Policy Schema

Schema file:
- `configs/policy/policy.schema.json`

Config file:
- `configs/policy/policy.json`

Single schema version source:
- `PolicySchemaVersion` in `src/schema.rs`

Rules:
- `schema_version` in config must match schema `const`.
- Unknown keys are rejected (`deny_unknown_fields` + top-level strict checks).
- Every documented default must include both `field` and `reason`.
