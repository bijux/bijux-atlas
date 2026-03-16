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

## Purpose

This page defines the Atlas contract expectations for artifact and store contracts. Use it when you need the explicit compatibility promise rather than a workflow narrative.

## Stability

This page is part of the checked-in contract surface. Changes here should stay aligned with tests, generated artifacts, and release evidence.
