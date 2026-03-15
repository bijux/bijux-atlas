# Schema registry configs

- Owner: `platform`
- Purpose: hold schema registry metadata and generated schema indexes.
- Consumers: schema indexing, config schema validation, and compatibility checks.
- Update workflow: update schema metadata and regenerated indexes together, then rerun schema and configs contracts.
- Boundary: schema versioning policy lives in `configs/registry/schema-versioning-policy.json` and is validated by this directory.
