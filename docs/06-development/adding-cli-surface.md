---
title: Adding CLI Surface
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Adding CLI Surface

New CLI surface should feel like it belongs to Atlas, not like a side entrance.

## CLI Addition Flow

```mermaid
flowchart TD
    Need[Need new CLI capability] --> Family[Choose the right command family]
    Family --> Handler[Implement behavior]
    Handler --> Output[Define stable output]
    Output --> Docs[Document surface]
    Docs --> Tests[Test help and behavior]
```

## Placement Model

```mermaid
flowchart LR
    CLIIntent[CLI intent] --> Inbound[adapters inbound cli]
    Inbound --> App[app use case or orchestration]
    App --> Contracts[contract-owned output if stable]
```

## Rules

- prefer extending the right command family over inventing a new miscellaneous root
- keep CLI parsing in inbound CLI adapters
- move reusable behavior into app or domain code when appropriate
- document stable output behavior if users or automation will depend on it

## Purpose

This page explains the Atlas material for adding cli surface and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
