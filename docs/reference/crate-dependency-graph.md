---
title: Crate dependency graph
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - crates
related:
  - docs/reference/crates.md
---

# Crate dependency graph

```mermaid
graph TD
  core[bijux-atlas-core]
  model[bijux-atlas-model]
  ingest[bijux-atlas-ingest]
  store[bijux-atlas-store]
  query[bijux-atlas-query]
  api[bijux-atlas-api]
  cli[bijux-atlas]
  server[bijux-atlas-server]
  policies[bijux-atlas-policies]

  model --> core
  ingest --> core
  ingest --> model
  store --> core
  store --> model
  query --> core
  query --> model
  query --> store
  query --> policies
  api --> core
  api --> model
  api --> query
  cli --> core
  cli --> model
  cli --> ingest
  cli --> store
  cli --> query
  cli --> policies
  server --> api
  server --> core
  server --> model
  server --> query
  server --> store
```

This graph reflects workspace crate-level dependencies, not third-party crates.
