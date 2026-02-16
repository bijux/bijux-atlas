# Effects

Allowed effects:

- Spawn `atlas-server` for `atlas serve`.
- Read input files and write output artifacts for `atlas ingest`.
- Read sqlite/manifest/catalog artifacts for validation and inspection commands.
- Spawn `atlas-openapi` for deterministic OpenAPI generation.

Forbidden effects in command planning and parsing code:

- Hidden network calls in unit-test paths.
