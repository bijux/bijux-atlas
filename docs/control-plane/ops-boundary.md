# Ops Boundary

- Owner: `ops-platform`

## Rule
- `ops/` contains operational data, manifests, runbooks, and low-level execution adapters.
- `bijux dev atlas` commands are the only supported control-plane entrypoints for executing ops workflows.
- Workflows and user docs must call `bijux dev atlas ops ...` (or thin `make` wrappers), never raw `ops/**` scripts.

## What Stays In `ops/`
- `ops/manifests/**`: declarative task manifests (data-only).
- `ops/schema/**`: schemas and contracts.
- `ops/_meta/**`: generated inventories and metadata.
- `bijux dev atlas ops ...` and thin `make` wrappers: supported operator entrypoints.
- `ops/vendor/**`: third-party or compatibility checks.

## What Moves To `bijux dev atlas`
- Orchestration, task discovery, and execution policy.
- User-facing command semantics and stable UX.
- Ownership and docs metadata for executable tasks.

## Canonical Entrypoints
- `bijux dev atlas ops run <task>`
- `bijux dev atlas ops list tasks`
- `bijux dev atlas ops explain <task>`
