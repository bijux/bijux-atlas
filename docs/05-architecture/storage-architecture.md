---
title: Storage Architecture
audience: maintainer
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Storage Architecture

Storage architecture in Atlas separates build output, serving store state, and transient runtime cache behavior.

## Storage Layers

```mermaid
flowchart TD
    BuildRoot[Build root] --> Publish[Publish]
    Publish --> ServingStore[Serving store]
    ServingStore --> Runtime[Runtime access]
    Runtime --> Cache[Transient cache]
```

## Durable vs Transient

```mermaid
flowchart LR
    Durable[Durable state] --> Store[Serving store and catalog]
    Transient[Transient state] --> Cache[Cache and in-memory acceleration]
```

## Architectural Rules

- build roots are validated outputs, not serving truth
- serving stores hold published artifacts and catalog state
- caches accelerate reads but do not redefine durable truth

## Why This Separation Matters

Without these storage boundaries, it becomes too easy to:

- point the runtime at the wrong directory
- confuse publication state with build state
- debug cache symptoms as if they were store corruption

