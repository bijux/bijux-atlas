# Architecture

## Architecture

`bijux-atlas-core` is the deterministic utility base layer.

Responsibilities:
- Canonical bytes/hash/cursor primitives.
- Shared machine error model and error codes.
- Environment-based config-path resolution helpers.

Non-responsibilities:
- Domain models.
- Ingestion/query logic.
- Runtime I/O integrations.
