---
title: Repository Home
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Repository

The repository handbook is the product-facing Atlas handbook for
`bijux-atlas`.

It will hold the deep documentation for the runtime package itself:

- product identity and boundaries
- ingest, dataset, and query workflows
- API and runtime interfaces
- source layout and runtime architecture
- published contracts for downstream users

```mermaid
flowchart LR
    Repository["Repository"]
    Repository --> Product["bijux-atlas"]
    Product --> Foundations["Foundations"]
    Product --> Workflows["Workflows"]
    Product --> Interfaces["Interfaces"]
    Product --> Runtime["Runtime"]
    Product --> Contracts["Contracts"]
```

## Scope

Use this handbook when the question is about what Atlas does as a product,
how users and integrators interact with it, and which runtime promises are
intended to stay stable.

## What Comes Next

The repository handbook is being rebuilt around `repository/bijux-atlas/`
with five durable subdirectories so the Atlas product surface can carry more
depth without mixing in maintainer-only or operations-only material.

## Current Paths

The first active repository slice is `repository/bijux-atlas/foundations/`.
It establishes the conceptual model for the runtime package before the
workflow, interface, runtime, and contract slices are migrated.

The next active slice is `repository/bijux-atlas/workflows/`, which carries
the user-facing Atlas product flows for install, ingest, dataset preparation,
server startup, and first queries.
