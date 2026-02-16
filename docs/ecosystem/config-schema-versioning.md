# Bijux Config Schema Versioning

- Each subsystem must expose `config_schema_version` as a stable string.
- Version increments follow:
  - patch: typo/comment-only docs fixes
  - minor: backward-compatible new keys
  - major: breaking key semantics/removal
- Runtime version endpoint must include schema version for operator visibility.
- Config migrations must be documented before version bump.
