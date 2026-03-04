---
title: Control Plane Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Control Plane Diagram

```mermaid
flowchart TB
    CLI[CLI Entry] --> REG[Registry Loader]
    REG --> ENG[Execution Engine]
    ENG --> CHK[Checks]
    ENG --> CTR[Contracts]
    ENG --> CMD[Command Runtime]
    CMD --> ART[Artifact Writers]
```
