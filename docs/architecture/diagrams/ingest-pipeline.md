---
title: Ingest Pipeline Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Ingest Pipeline Diagram

```mermaid
flowchart LR
    A[Input Discovery] --> B[Schema Validation]
    B --> C[Normalization]
    C --> D[Quality Gates]
    D --> E[Manifest Assembly]
    E --> F[Versioned Artifact Output]
```
