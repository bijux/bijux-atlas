---
title: Runtime System Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Runtime System Diagram

```mermaid
flowchart LR
    API[API Server] --> QRY[Query Layer]
    QRY --> IDX[Dataset Index]
    QRY --> ART[Artifact Storage]
    API --> MET[Metrics and Health]
```
