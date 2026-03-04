---
title: Data Flow Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Data Flow Diagram

```mermaid
flowchart LR
    S[Source Data] --> I[Ingest]
    I --> M[Manifest]
    I --> A[Artifacts]
    A --> Q[Query]
    A --> O[Ops]
    O --> E[Evidence]
    A --> R[Release]
    R --> B[Bundle]
```
