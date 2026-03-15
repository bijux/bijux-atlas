---
title: Adding HTTP Surface
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Adding HTTP Surface

New HTTP surface should preserve the separation between routing, policy, execution, and presentation.

## HTTP Addition Flow

```mermaid
flowchart TD
    Need[Need new endpoint] --> Route[Add route]
    Route --> Handler[Add handler path]
    Handler --> App[Call app and domain logic]
    App --> Contract[Update contract and OpenAPI if needed]
    Contract --> Tests[Add interface tests]
```

## Layering Model

```mermaid
flowchart LR
    Router[Router] --> Middleware[Middleware and policy]
    Middleware --> Handler[Handler]
    Handler --> App[App service]
    App --> Store[Store or query backend]
```

## Rules

- keep router declarations declarative
- keep HTTP concerns in HTTP adapters
- avoid letting HTTP types become application truth
- update documentation and contracts when the surface is stable or public

