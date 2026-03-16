---
title: Query Architecture
audience: maintainer
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Query Architecture

Atlas query architecture is shaped around explicit dataset identity, request classification, policy checks, and store-backed execution.

## Query Path

```mermaid
flowchart LR
    Request[Query request] --> Validate[Validate and classify]
    Validate --> Dataset[Resolve dataset]
    Dataset --> Execute[Execute against store-backed state]
    Execute --> Present[Present response]
```

This query path highlights the stages Atlas wants to keep visible: request classification, dataset
resolution, execution, and presentation. That visibility is what makes rejection reasons and result
shape easier to explain.

## Query Concerns

```mermaid
mindmap
  root((Query architecture))
    Dataset identity
    Query classification
    Cost estimation
    Policy enforcement
    Result presentation
```

This concern map shows the five themes that shape most query changes. It helps maintainers avoid
mixing request transport, cost policy, and result presentation into one indistinct layer.

## Architectural Priorities

- explicit selectors beat implicit scans
- policy should explain rejection clearly
- response structure should remain deterministic
- query logic should not leak transport concerns into domain rules

## Why Query Validation Exists

The dedicated validation route is not just a convenience. It exposes the classification and policy
model directly so clients can understand request shape without needing to infer behavior from full
execution only.

## Healthy Query Architecture Traits

- explicit dataset identity remains part of the request contract
- policy rejection is explainable before expensive execution starts
- presentation does not smuggle transport concerns into core query rules

## Purpose

This page explains the Atlas material for query architecture and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
