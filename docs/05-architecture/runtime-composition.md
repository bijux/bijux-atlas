---
title: Runtime Composition
audience: maintainer
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Runtime Composition

Runtime composition is the process of turning Atlas modules into a running server process with concrete configuration, limits, backends, and middleware.

## Composition Model

```mermaid
flowchart LR
    Config[Runtime config] --> Runtime[Runtime composition]
    App[App services] --> Runtime
    Adapters[Concrete adapters] --> Runtime
    Runtime --> Server[Running server]
```

This composition model explains what runtime owns: taking abstract services, concrete adapters, and
configuration and turning them into one running process.

## Runtime Responsibilities

```mermaid
flowchart TD
    Runtime[Runtime] --> BuildState[Construct app state]
    Runtime --> Router[Assemble router and middleware]
    Runtime --> Limits[Apply limits and policy mode]
    Runtime --> Backends[Choose concrete backends]
```

This responsibility map is useful because runtime composition often grows opportunistically over
time. The diagram keeps the intended boundary visible for future changes.

## Architectural Boundary

Runtime is where concrete choices belong:

- addresses and bind settings
- store and cache roots
- concurrency and rate-limiting settings
- telemetry backends

Those choices should not leak backward and become domain rules.

## Runtime Composition Smell Test

- are you wiring concrete behavior or redefining domain meaning?
- are you selecting a backend or sneaking infrastructure detail into a contract?
- are you composing limits and middleware where operators expect them to live?

## Purpose

This page explains the Atlas material for runtime composition and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
