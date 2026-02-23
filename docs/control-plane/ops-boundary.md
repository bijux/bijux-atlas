# Ops Boundary

- Owner: `ops-platform`

## Rule
- `ops/` contains operational data, manifests, runbooks, and low-level execution adapters.
- `atlasctl` commands are the only supported control-plane entrypoints for executing ops workflows.
- Workflows and user docs must call `./bin/atlasctl ops ...` (or thin `make` wrappers), never raw `ops/**` scripts.

## What Stays In `ops/`
- `ops/manifests/**`: declarative task manifests (data-only).
- `ops/schema/**`: schemas and contracts.
- `ops/_meta/**`: generated inventories and metadata.
- `atlasctl ops ...` and thin `make` wrappers: supported operator entrypoints.
- `ops/vendor/**`: third-party or compatibility checks.

## What Moves To `atlasctl`
- Orchestration, task discovery, and execution policy.
- User-facing command semantics and stable UX.
- Ownership and docs metadata for executable tasks.

## Canonical Entrypoints
- `./bin/atlasctl ops run <task>`
- `./bin/atlasctl ops list tasks`
- `./bin/atlasctl ops explain <task>`
