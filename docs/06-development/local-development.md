---
title: Local Development
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Local Development

Local development should make it easy to iterate without teaching bad habits.

## Local Development Loop

```mermaid
flowchart TD
    Edit[Edit code or docs] --> Build[Build or run focused checks]
    Build --> Test[Run tests]
    Test --> Docs[Update docs if surface changed]
    Docs --> Review[Review ownership and contracts]
```

## Local Environment Model

```mermaid
flowchart LR
    Source[Workspace source] --> Artifacts[artifacts/]
    Source --> Fixtures[Test fixtures]
    Fixtures --> LocalRuns[Local runs and validation]
```

## Safe Local Habits

- keep local outputs in `artifacts/`
- use committed fixtures for reproducible local experiments
- validate the layer you changed instead of only running a giant command blindly
- preserve the canonical module ownership model when moving code

## Purpose

This page explains the Atlas material for local development and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
