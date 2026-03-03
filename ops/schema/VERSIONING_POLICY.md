# Schema Versioning Policy

- Owner: `bijux-atlas-operations`
- Purpose: keep ops schema evolution explicit and backward-compatible by default

All ops schemas require `schema_version`.
Breaking changes must update the compatibility lock and be reviewed intentionally.
