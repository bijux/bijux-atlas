---
title: Operations Home
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Operations

The operations handbook is the operating surface for `bijux-atlas-ops`.

Atlas has a large operational footprint across `ops/`, `ops/k8s/`,
`ops/stack/`, `ops/observe/`, `ops/load/`, `ops/release/`, and surrounding
policy and report surfaces. This handbook exists so that depth has a real home
instead of being compressed into a small generic section.

```mermaid
flowchart LR
    Operations["Operations"]
    Operations --> Stack["Stack and environment"]
    Operations --> Kubernetes["Kubernetes and Helm"]
    Operations --> Release["Rollout and release"]
    Operations --> Observe["Observability and drills"]
    Operations --> Load["Load, stress, and resilience"]
```

## Scope

Use this handbook when the question is about operating Atlas safely:
deployment profiles, Helm values, Kubernetes validation, stack dependencies,
release drills, observability, or load execution.

## What Comes Next

The operations handbook is being rebuilt around `operations/bijux-atlas-ops/`
with five durable subdirectories so Kubernetes, Helm, stress, and operating
evidence can be documented at the depth the repository actually needs.
