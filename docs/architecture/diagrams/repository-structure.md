---
title: Repository Structure Diagram
audience: contributor
type: reference
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Repository Structure Diagram

```mermaid
flowchart TB
    ROOT[Repository Root] --> CR[crates]
    ROOT --> CFG[configs]
    ROOT --> DOC[docs]
    ROOT --> OPS[ops]
    ROOT --> REL[release]
    ROOT --> ART[artifacts]
```
