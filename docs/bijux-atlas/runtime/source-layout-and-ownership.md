---
title: Source Layout and Ownership
audience: maintainer
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Source Layout and Ownership

Atlas source layout is meant to teach the architecture directly from the tree,
without relying on tribal knowledge.

## Canonical Ownership Model

```mermaid
flowchart LR
    Adapters[src/adapters] --> Inbound[CLI and HTTP]
    Adapters --> Outbound[Store sqlite redis fs telemetry]
    App[src/app] --> UseCases[Use cases and ports]
    Contracts[src/contracts] --> Stable[Stable schemas and errors]
    Domain[src/domain] --> Rules[Business rules]
    Runtime[src/runtime] --> Wiring[Runtime config and composition]
```

This ownership map gives contributors a direct translation from architectural
idea to source-tree location. New code placement should not depend on old
directory names or local folklore.

## Why These Roots Exist

```mermaid
flowchart TD
    Domain[Pure rules] --> App[Orchestration]
    App --> Adapters[Integrations]
    Contracts --> Domain
    Contracts --> Adapters
    Runtime --> App
    Runtime --> Adapters
```

This diagram explains the dependency intent behind the root layout. The point is
not just tidy directories; it is making responsibility and change impact easier
to reason about.

The canonical roots are:

- `adapters`
- `app`
- `contracts`
- `domain`
- `runtime`

These are the roots contributors should optimize for when placing new code.

## Ownership Rules

- if the code translates between Atlas and the outside world, it belongs in `adapters`
- if the code orchestrates use cases and ports, it belongs in `app`
- if the code defines stable schemas, config contracts, or errors, it belongs in `contracts`
- if the code defines business rules and domain semantics, it belongs in `domain`
- if the code wires the process together, it belongs in `runtime`

## Architectural Benefit

This layout makes it harder to hide source of truth behind historical barrels or convenience shims.

## Quick Placement Test

- if you are translating HTTP, CLI, filesystem, or network concerns, start in `adapters`
- if you are composing use cases or ports, start in `app`
- if you are defining stable promises, start in `contracts`
- if you are defining rules that should not depend on transport, start in `domain`
- if you are wiring concrete runtime behavior, start in `runtime`

## Reading Rule

Use this page when a change feels reasonable in concept but you still cannot
tell which root should own it.
