---
title: Shard Routing Visualization
audience: contributor
type: reference
stability: evolving
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - diagrams
  - routing
---

# Shard Routing Visualization

```mermaid
flowchart TD
    K[Route Key] --> H[Stable Hash]
    H --> IDX[Shard Index]
    IDX --> OWN[Owner Node]
    OWN --> RES[Shard Result]
```
