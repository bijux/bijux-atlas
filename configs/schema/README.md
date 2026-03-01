# Schema registry configs

- Owner: `platform`
- Purpose: hold schema registry metadata, generated schema indexes, and versioning policy inputs.
- Consumers: schema indexing, config schema validation, and compatibility checks.
- Update workflow: update schema metadata and regenerated indexes together, then rerun schema and configs contracts.
