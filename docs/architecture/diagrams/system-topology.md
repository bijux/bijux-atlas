---
title: System Topology Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# System Topology Diagram

```mermaid
flowchart TB
    SRC[Source Providers] --> ING[Ingest Workers]
    ING --> ART[Artifact Store]
    ART --> QRY[Query Runtime]
    ART --> OPS[Ops Runtime]
    OPS --> REL[Release Assembly]
    REL --> AUD[Audit Bundle]
    CP[Control Plane] --> ING
    CP --> QRY
    CP --> OPS
    CP --> REL
```
