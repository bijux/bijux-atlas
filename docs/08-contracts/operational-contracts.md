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

This operational-scope diagram shows the stable operator-facing surfaces Atlas expects deployments to
rely on intentionally.

## Operator Promise Model

```mermaid
flowchart TD
    Promise[Operational promise] --> Checks[Checks and evidence]
    Checks --> Deploy[Safe deployment use]
```

This promise model explains how operational contracts should stay credible: they must connect to
checks and evidence that operators can actually use during deployment and recovery.

## Main Promise Areas

- health and readiness semantics
- metrics and observability surfaces
- runtime validation behavior
- explicit operator-visible error conditions

## Purpose

This page defines the Atlas contract expectations for operational contracts. Use it when you need the explicit compatibility promise rather than a workflow narrative.

## Stability

This page is part of the checked-in contract surface. Changes here should stay aligned with tests, generated artifacts, and release evidence.
