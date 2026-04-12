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

This CLI addition flow keeps maintainers from treating new commands as isolated parser work. A new
CLI surface changes behavior, output, docs, and tests together.

## Placement Model

```mermaid
flowchart LR
    CLIIntent[CLI intent] --> Inbound[adapters inbound cli]
    Inbound --> App[app use case or orchestration]
    App --> Contracts[contract-owned output if stable]
```

This placement model explains where CLI-specific concerns should stop. Parsing belongs in inbound
adapters, while reusable logic should move deeper into app or domain layers.

## Rules

- prefer extending the right command family over inventing a new miscellaneous root
- keep CLI parsing in inbound CLI adapters
- move reusable behavior into app or domain code when appropriate
- document stable output behavior if users or automation will depend on it

## CLI Surface Check Before Merge

- does the command belong in an existing family?
- does help output still tell an honest story?
- is any stable output now contract-worthy?

## Purpose

This page explains the Atlas material for adding cli surface and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
