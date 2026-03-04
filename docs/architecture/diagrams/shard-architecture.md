---
title: Shard Architecture
audience: contributor
type: reference
stability: evolving
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - diagrams
  - sharding
---

# Shard Architecture

```mermaid
flowchart LR
    Q[Query Request] --> R[Shard Router]
    R --> S1[Shard A Owner]
    R --> S2[Shard B Owner]
    R --> S3[Shard C Owner]
    S1 --> M[Shard Registry]
    S2 --> M
    S3 --> M
    M --> MET[/metrics atlas_shard_*/]
```
