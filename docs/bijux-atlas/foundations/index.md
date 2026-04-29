---
title: Foundations
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Foundations

`bijux-atlas/foundations` explains the product model that the rest of Atlas
builds on.

```mermaid
flowchart TD
    Foundations[Foundations section] --> Identity[What Atlas is]
    Foundations --> Boundaries[Boundaries and non-goals]
    Foundations --> Concepts[Core concepts]
    Foundations --> Stability[Guarantees and stability]
    Identity --> Shared[Shared architectural language]
    Boundaries --> Shared
    Concepts --> Shared
    Stability --> Shared
```

Foundations is where Atlas stops being a list of files and starts becoming a
coherent product model. These pages define the terms and boundaries the rest of
Atlas relies on.

Use this section when you are trying to answer:

- what Atlas is actually for
- which product boundaries are intentional
- which concepts and terms matter before you read exact interfaces
- how datasets, queries, releases, and stability fit together

## Recommended Reading Order

Read these pages in this order when you are new to Atlas:

1. [What Atlas Is](what-atlas-is.md)
2. [Core Concepts](core-concepts.md)
3. [Boundaries and Non-Goals](boundaries-and-non-goals.md)
4. [Guarantees and Stability](guarantees-and-stability.md)

After that, use the remaining pages as targeted follow-ups for specific
product-model questions.

## What This Section Covers

- product identity and repository fit
- the conceptual model for datasets, releases, and query behavior
- the difference between documented promises and current implementation detail
- the handoff from product foundations into workflows, interfaces, runtime, and contracts

## Boundary For This Section

This section may define terms, architectural boundaries, and stability posture.
It should not become a duplicate command reference, an API index, or an ops
runbook. Once the question turns into exact runtime behavior or an exact
user-facing surface, move on.

## Pages

- [Boundaries and Non-Goals](boundaries-and-non-goals.md)
- [Core Concepts](core-concepts.md)
- [Dataset Model](dataset-model.md)
- [Documentation Map](documentation-map.md)
- [Guarantees and Stability](guarantees-and-stability.md)
- [Package Ownership](package-ownership.md)
- [Query Model](query-model.md)
- [Release Model](release-model.md)
- [Runtime Surfaces](runtime-surfaces.md)
- [What Atlas Is](what-atlas-is.md)

## What You Should Know Before Leaving

Leave this section once you can answer three questions clearly:

- what counts as Atlas product behavior versus operations or repository-governance behavior
- what the runtime is serving and why artifacts matter
- which surfaces are strong compatibility promises and which are only explanatory
