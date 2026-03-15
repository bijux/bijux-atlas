---
title: Cache and Store Operations
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Cache and Store Operations

Atlas has strong opinions about store and cache behavior because durable release state and transient performance state should not blur together.

## Store vs Cache

```mermaid
flowchart LR
    Store[Serving store] --> Durable[Durable release state]
    Cache[Runtime cache] --> Transient[Transient acceleration state]
```

The store is durable and authoritative for serving content.

The cache is disposable and performance-oriented.

## Operational Model

```mermaid
flowchart TD
    Published[Published artifacts and catalog] --> Store[Serving store]
    Store --> Runtime[Runtime lookup path]
    Runtime --> Cache[Cache warm or populate]
    Cache --> Response[Serve response faster]
```

## Operator Rules

- never treat cache contents as the durable source of truth
- keep cache roots under the sanctioned artifacts hierarchy
- understand what happens when caches are cold or unavailable
- validate store integrity before assuming query failures are only cache-related

## Practical Questions

- is the store root complete and discoverable?
- is `catalog.json` present and correct?
- is cache growth bounded and expected?
- can the service recover safely from cache loss?

## Failure Interpretation

If a cold cache makes responses slower, that is usually a performance issue.

If the store is missing or inconsistent, that is a correctness and availability issue.

