# Build/Serve Split

- Owner: `atlas-architecture`
- Stability: `stable`

## Rule

Build-time and serve-time concerns are separate contracts.

- Build-time: ingest and publish immutable artifacts.
- Serve-time: read immutable artifacts, serve APIs, maintain cache/index state.

## Why

Mixing build and serve concerns introduces nondeterminism, mutable state coupling, and rollback ambiguity.

## Enforcement

- `make architecture-check` enforces dependency and boundary guards.
- Server opens SQLite artifacts read-only with `immutable=1` and `query_only=ON`.
- Server must not depend on ingest crate internals.
