---
title: Upgrades and Rollback
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Upgrades and Rollback

Atlas upgrades should preserve two invariants:

- contract-owned surfaces remain understood and validated
- serving state stays recoverable if a rollout goes wrong

## Upgrade Flow

```mermaid
flowchart TD
    Validate[Validate runtime and contracts] --> Rollout[Roll out new runtime]
    Rollout --> Observe[Observe health and load]
    Observe --> Keep[Keep rollout]
    Observe --> Rollback[Rollback if needed]
```

This upgrade flow keeps rollout discipline visible. Atlas upgrades should be validated, observed,
and explicitly kept or rolled back rather than treated as one-way jumps.

## Rollback Flow

```mermaid
flowchart LR
    Problem[Operational problem] --> Scope[Determine runtime vs store scope]
    Scope --> RuntimeRollback[Rollback runtime]
    Scope --> StoreRollback[Rollback serving state if required]
```

This rollback flow explains one of the most important operator distinctions in Atlas: not every
incident needs store-state rollback, and not every rollback should start there.

## Operator Guidance

- separate runtime rollback from store-state rollback in your thinking
- verify health, readiness, and key query paths after rollout
- keep rollback paths explicit before you need them
- use compatibility and contract evidence as rollout input, not only hope and manual spot checks

## What to Watch During Upgrade

- readiness instability
- unusual rejection or error patterns
- metrics or traces indicating saturation changes
- catalog or dataset discoverability regressions

## Rollout Question That Saves Time

Ask first whether the change affected runtime behavior, serving-store state, or both. That answer
usually determines the safest rollback path.

## Purpose

This page explains the Atlas material for upgrades and rollback and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
