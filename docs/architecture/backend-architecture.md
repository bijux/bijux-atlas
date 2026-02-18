# Backend Architecture

- Owner: `atlas-server`
- Stability: `stable`

## Runtime Graph

`API handlers -> query planner/executor -> dataset cache manager -> store backends -> immutable artifacts`

- API layer (`bijux-atlas-server` + `bijux-atlas-api`) parses/validates requests and maps transport concerns.
- Query layer (`bijux-atlas-query`) is pure query logic and SQL planning; it does not perform network/runtime orchestration.
- Cache layer (`DatasetCacheManager`) owns local dataset materialization, integrity checks, and read-only SQLite handles.
- Store layer (`bijux-atlas-store` and server store adapters) resolves catalogs/manifests/artifacts from fs/http/s3-like sources.
- Artifact layer is immutable release-indexed content (`manifest + sqlite + auxiliary files`).

## Invariants

- Serving path is read-only for artifact SQLite files.
- Server crate must not depend on ingest internals.
- Query crate remains runtime-pure (no `tokio/reqwest/axum/hyper` dependencies).
- Policy/config is canonicalized at startup and hashed (`runtime_policy_hash`).

## Build/Serve Split

- Build/ingest concerns live in ingest/ops flows.
- Serve concerns live in server/query/store runtime path.
- Runtime never mutates build artifacts; it only caches and reads them.

## How To Verify

```bash
make architecture-check
cargo test -p bijux-atlas-server --test api-contracts
```
