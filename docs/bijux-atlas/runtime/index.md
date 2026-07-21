---
title: Runtime
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Runtime

The Atlas runtime resolves immutable dataset releases and executes bounded
queries through CLI and HTTP delivery surfaces. It composes domain policy,
application use cases, store adapters, caches, configuration, security, and
telemetry without moving release authority into a running process.

```mermaid
flowchart LR
    Config[Validated runtime configuration] --> Compose[Runtime composition]
    Catalog[Published catalog state] --> Compose
    Store[Immutable artifact store] --> Compose
    Compose --> CLI[CLI delivery]
    Compose --> HTTP[HTTP delivery]
    CLI --> Query[Bounded query execution]
    HTTP --> Query
    Query --> Evidence[Structured result and telemetry]
```

## Runtime Invariants

- The catalog selects a published dataset identity; a request cannot invent one.
- The serving store supplies immutable artifact state; caches do not become
  release authority.
- Domain and query crates own biological and query semantics; delivery adapters
  do not redefine them.
- Configuration is resolved before it controls runtime behavior and remains
  attributable in diagnostics.
- Security, overload, and concurrency boundaries may reject work before query
  execution.
- Successful output preserves enough release and request context to be
  interpreted against the owning contract.

## Follow the Running System

| Question | Read |
| --- | --- |
| How do crates and control planes fit together? | [System Overview](system-overview.md) |
| Where is behavior owned in source? | [Source Layout and Ownership](source-layout-and-ownership.md) |
| How is the concrete runtime assembled? | [Runtime Composition](runtime-composition.md) |
| What happens from startup to shutdown? | [Runtime Process Model](runtime-process-model.md) |
| How does an HTTP request cross boundaries? | [Request Lifecycle](request-lifecycle.md) |
| How does source data become a release? | [Ingest Architecture](ingest-architecture.md) and [Artifact Lifecycle](artifact-lifecycle.md) |
| How are queries planned and executed? | [Query Architecture](query-architecture.md) |
| Which storage boundary is authoritative? | [Storage Architecture](storage-architecture.md) and [Serving Store Model](serving-store-model.md) |

## Architecture, Interface, and Operations

Runtime architecture explains ownership and execution flow. Exact flags,
environment variables, HTTP paths, output shapes, and error codes belong to
[Interfaces](../interfaces/index.md). Compatibility guarantees belong to
[Contracts](../contracts/index.md). Deployment profiles, observability,
capacity, recovery, and release decisions belong to the
[Operations handbook](../../bijux-atlas-ops/index.md).

The distinction matters during failure analysis. A runtime explanation can
identify the rejecting boundary, but only its interface contract defines the
consumer-visible result. Only captured operational evidence establishes what
happened in a particular environment.
