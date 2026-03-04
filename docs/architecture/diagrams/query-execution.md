---
title: Query Execution Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Query Execution Diagram

```mermaid
flowchart LR
    RQ[Request] --> PV[Parameter Validation]
    PV --> PL[Query Planning]
    PL --> EX[Artifact-backed Execution]
    EX --> RS[Result Shaping]
    RS --> PG[Pagination and Cursor]
    PG --> RP[Response]
```
