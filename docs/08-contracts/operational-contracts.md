---
title: Operational Contracts
audience: operator
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Operational Contracts

Operational contracts define the stable expectations operators can rely on around health, readiness, observability, and runtime behavior.

## Operational Contract Scope

```mermaid
flowchart LR
    Ops[Operational contracts] --> Health[Health and readiness]
    Ops --> Metrics[Metrics]
    Ops --> Runtime[Runtime validation]
```

## Operator Promise Model

```mermaid
flowchart TD
    Promise[Operational promise] --> Checks[Checks and evidence]
    Checks --> Deploy[Safe deployment use]
```

## Main Promise Areas

- health and readiness semantics
- metrics and observability surfaces
- runtime validation behavior
- explicit operator-visible error conditions

