# Bijux Atlas Crate Boundaries

Concept ID: `concept.crate-boundaries`

- Owner: `bijux-atlas-core`

Allowed crate dependency directions:

- `bijux-atlas-core`: no internal dependencies.
- `bijux-atlas-model` -> `bijux-atlas-core`.
- `bijux-atlas-ingest` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-store` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-query` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-store`, `bijux-atlas-policies`.
- `bijux-atlas-api` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`.
- `bijux-atlas-cli` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-ingest`, `bijux-atlas-store`, `bijux-atlas-query`, `bijux-atlas-policies`.
- `bijux-atlas-server` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-api`, `bijux-atlas-store`, `bijux-atlas-query`.

```mermaid
flowchart LR
  core[core]
  model[model]
  ingest[ingest]
  store[store]
  query[query]
  api[api]
  cli[cli]
  server[server]

  model --> core
  ingest --> core
  ingest --> model
  store --> core
  store --> model
  query --> core
  query --> model
  query --> store
  api --> core
  api --> model
  api --> query
  cli --> core
  cli --> model
  cli --> ingest
  cli --> store
  cli --> query
  server --> core
  server --> model
  server --> api
  server --> store
  server --> query
```

Disallowed by default:

- Any dependency edge not listed above.
- Cycles among internal crates.
- `bijux-atlas-server` importing ingest internals directly.
- Query runtime dependencies (`tokio`, `reqwest`, `axum`, `hyper`) in `bijux-atlas-query`.
