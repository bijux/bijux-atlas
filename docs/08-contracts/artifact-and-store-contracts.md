---
title: Artifact and Store Contracts
audience: mixed
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Artifact and Store Contracts

Artifact and store contracts define how durable dataset state is shaped and what the runtime expects to discover.

## Contracted Storage Shape

```mermaid
flowchart LR
    Build[Build root] --> Publish[Publish]
    Publish --> Store[Serving store]
    Store --> Catalog[catalog.json]
```

## Contract Focus

```mermaid
flowchart TD
    ArtifactContract[Artifact contract] --> Manifest[Manifest shape]
    ArtifactContract --> Sqlite[SQLite artifact expectations]
    ArtifactContract --> Catalog[Catalog discoverability]
```

## Main Promise

Atlas should make the durable serving shape explicit enough that publication, serving, backup, and recovery can all reason about the same artifact model.

