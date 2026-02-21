# Output Breaking Change Policy

This policy governs JSON output contracts used by `atlasctl`.

## Rule

- Stable command outputs must not introduce breaking schema changes in-place.
- Breaking changes require a new schema version suffix (`.v2`, `.v3`, ...).
- `catalog.json` remains the single source of truth for schema ids and versions.

## Required Gates

- `atlasctl contracts validate --report json`
- stable command compatibility checks
- schema/golden validation checks in the suite

## Migration

- Keep previous stable schema available while consumers migrate.
- Document the migration in release notes with explicit version/date.
