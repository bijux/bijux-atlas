---
title: Backup and Recovery
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Backup and Recovery

Atlas recovery planning should focus on the durable serving store and the ability to reconstruct runtime state safely.

## Recovery Priority

```mermaid
flowchart TD
    Recover[Recovery planning] --> Store[Serving store]
    Recover --> Catalog[Catalog state]
    Recover --> Runtime[Runtime config]
    Recover --> Cache[Cache state if useful]
```

This recovery-priority diagram keeps the durable pieces at the center. Atlas recovery should start
from serving store state, catalog state, and runtime configuration before anyone worries about cache
warmth.

## What Matters Most

- published manifests and SQLite artifacts
- catalog state that exposes those published datasets
- the runtime configuration needed to serve them correctly

## Recovery Model

```mermaid
flowchart LR
    Backup[Backed up store and config] --> Restore[Restore store root and config]
    Restore --> Validate[Validate discoverability and readiness]
    Validate --> Serve[Resume service]
```

This recovery model emphasizes validation after restore. A restored file tree is not yet a recovered
service until discoverability and readiness checks say so.

## Practical Advice

- back up the serving store, not only a build root
- treat catalog integrity as part of recoverability
- keep recovery procedures separate from cache rewarming procedures
- verify readiness after restore rather than assuming successful file copy equals successful service recovery

## What Recovery Is Not

Recovery is not “copy whatever is in the cache and hope for the best.” Cache loss may hurt performance, but store loss is what threatens durable serving ability.

## Recovery Questions to Answer Before an Incident

- where is the authoritative backup of the serving store?
- how is catalog integrity preserved or rebuilt?
- what checks prove the recovered instance is ready to serve again?

## Purpose

This page explains the Atlas material for backup and recovery and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
