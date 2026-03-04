---
title: Artifact Lifecycle Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Artifact Lifecycle Diagram

```mermaid
flowchart LR
    C[Create] --> V[Validate]
    V --> P[Promote]
    P --> R[Release Bundle]
    R --> O[Operate]
    O --> T[Retain or Retire]
```
