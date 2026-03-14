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
  api[bijux-atlas-api]
  cli[bijux-atlas]
  server[bijux-atlas-server]

  ingest --> core
  ingest --> model
  store --> core
  store --> model
  api --> cli
  api --> core
  api --> model
  cli --> core
  cli --> model
  cli --> ingest
  cli --> store
  server --> api
  server --> cli
  server --> core
  server --> model
  server --> store
```

This graph reflects workspace crate-level dependencies, not third-party crates.
