---
title: Cache and Store Operations
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Cache and Store Operations

Atlas has strong opinions about store and cache behavior because durable release state and transient performance state should not blur together.

```mermaid
flowchart TD
    Request[Client request] --> Runtime[Atlas runtime]
    Runtime --> Hit{Cache hit?}
    Hit -- Yes --> Cache[Serve from Redis or cache layer]
    Hit -- No --> Store[Read durable store via MinIO or serving artifacts]
    Store --> Serve[Serve result]
    Serve --> Warm[Optionally warm cache]
    Durable[Durable serving state] -. must not be confused with .-> Transient[Transient acceleration state]
```

This page exists to keep two operator instincts separate: where Atlas keeps the
truth it serves, and where it keeps acceleration state that can be rebuilt. The
whole point of the cache/store model is to stop performance troubleshooting from
becoming accidental data-governance confusion.

## Store vs Cache

```mermaid
flowchart LR
    Store[Serving store] --> Durable[Durable release state]
    Cache[Runtime cache] --> Transient[Transient acceleration state]
```

This store-versus-cache diagram names the most important operational separation in Atlas. The store
is durable serving truth; the cache is optional performance help.

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

This operating model shows where the cache sits in the request path. It is downstream of the store,
which is why cache loss and store loss should be treated as different classes of incident.

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

## Operator Checks Worth Automating

- verify the serving store layout and catalog presence
- observe cache size, miss behavior, and recovery after cold start
- make sure cache loss does not look like data loss in your runbooks

## Purpose

This page explains the Atlas material for cache and store operations and points readers to the canonical checked-in workflow or boundary for this topic.

## Source of Truth

- `ops/stack/redis/redis.yaml`
- `ops/stack/minio/minio.yaml`
- `ops/k8s/values/documentation-map.json`
- `ops/observe/dashboards/atlas-artifact-cache-performance-dashboard.json`

## Concrete Component Mapping

- Redis is the declared cache component in `ops/stack/redis/redis.yaml`
- MinIO is the declared durable object store component in
  `ops/stack/minio/minio.yaml`
- the Kubernetes documentation map ties `cache`, `catalog`, and `store` keys to
  this operating topic
- cache behavior is reflected in the cache performance dashboard pack

## Main Takeaway

Cache loss is a performance event. Store loss is a durability and correctness
event. Good Atlas operations keep those ideas separate in values, dashboards,
incident response, and recovery procedures.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
