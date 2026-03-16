---
title: Ingest Architecture
audience: maintainer
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Ingest Architecture

Ingest is the architectural boundary between raw source inputs and validated Atlas build state.

## Ingest Pipeline

```mermaid
flowchart LR
    Inputs[GFF3 FASTA FAI] --> Validate[Input validation]
    Validate --> Normalize[Normalization]
    Normalize --> Derive[Derived artifacts]
    Derive --> BuildRoot[Build root]
```

This ingest pipeline shows why Atlas treats ingest as an architectural boundary. The layer receives
raw supported inputs and emits validated build state plus derived artifacts that later workflows can
trust more than the original files.

## Architectural Outcome

```mermaid
flowchart TD
    BuildRoot[Build root] --> Verify[Verify dataset]
    Verify --> Publish[Publish into serving store]
    Publish --> Serve[Serve through runtime]
```

This outcome diagram makes the stop point explicit. Ingest ends at the build root so validation,
publication, and runtime serving stay separate and reviewable.

## Why Ingest Stops at a Build Root

Atlas deliberately avoids making ingest directly equal to serving state. That separation enables:

- publication gates
- deterministic validation and verification
- explicit promotion into serving state

## What the Ingest Layer Owns

- input parsing and validation
- normalization and anomaly reporting
- derived artifacts such as manifests and SQLite summaries

It does not own:

- catalog discoverability
- runtime serving policy
- long-lived cache behavior

## Why This Boundary Saves Pain

- ingest bugs stay distinguishable from serving-store bugs
- publication remains an explicit gate instead of an implicit side effect
- runtime behavior does not have to compensate for half-defined ingest output

## Purpose

This page explains the Atlas material for ingest architecture and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
