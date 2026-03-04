---
title: Ops Governance Diagram
audience: operator
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Ops Governance Diagram

```mermaid
flowchart LR
    PRF[Ops Profile] --> RND[Render]
    RND --> VAL[Validation Gates]
    VAL --> EV[Evidence]
    EV --> REL[Release Gate]
    REL --> AUD[Audit Readiness]
```
