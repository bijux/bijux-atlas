# Schema Versioning Policy

- Every ops schema requires `schema_version` in `required` and `properties`.
- Breaking schema changes require coordinated updates to consumers and generated indexes.
- Regenerate indexes and compatibility lock after schema changes.
