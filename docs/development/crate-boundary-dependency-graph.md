# Crate Boundary Dependency Graph

This graph visualizes allowed dependency direction across Atlas crates.

```mermaid
graph TD
  core[bijux-atlas-core]
  model[bijux-atlas-model]
  policies[bijux-atlas-policies]
  ingest[bijux-atlas-ingest]
  store[bijux-atlas-store]
  query[bijux-atlas-query]
  api[bijux-atlas-api]
  server[bijux-atlas-server]
  cli[bijux-atlas-cli]

  model --> core
  policies --> core
  ingest --> core
  ingest --> model
  ingest --> query
  query --> core
  query --> model
  query --> policies
  api --> core
  api --> model
  api --> query
  store --> core
  store --> model
  server --> core
  server --> model
  server --> query
  server --> api
  server --> store
  server --> policies
  cli --> core
  cli --> model
  cli --> query
  cli --> api
  cli --> ingest
  cli --> store
  cli --> server
  cli --> policies
```

Source of truth for boundaries and guardrails:
- `docs/architecture/boundaries.md`
- `docs/architecture/effects.md`
- crate-level `docs/ARCHITECTURE.md`
