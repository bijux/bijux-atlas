---
title: Cluster Membership Architecture
audience: contributor
type: reference
stability: evolving
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - diagrams
  - membership
related:
  - docs/architecture/cluster-membership-lifecycle.md
---

# Cluster Membership Architecture

```mermaid
flowchart LR
    N[Node Runtime] -->|register| REG[/debug/cluster/register]
    N -->|heartbeat| HB[/debug/cluster/heartbeat]
    OP[Operator] --> MODE[/debug/cluster/mode]
    REG --> MR[(Membership Registry)]
    HB --> MR
    MODE --> MR
    MR --> NS[/debug/cluster/nodes]
    MR --> CS[/debug/cluster-status]
    MR --> MET[/metrics atlas_membership_*]
    MR --> LOG[Structured Event Logs]
```
