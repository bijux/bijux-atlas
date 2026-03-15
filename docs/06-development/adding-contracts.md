---
title: Adding Contracts
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Adding Contracts

Contracts are how Atlas turns intent into a stable, reviewable promise.

## Contract Addition Flow

```mermaid
flowchart TD
    Need[Need stable promise] --> Owner[Choose contract owner module]
    Owner --> Define[Define schema or rule]
    Define --> Docs[Document promise]
    Docs --> Tests[Add contract tests]
```

## Ownership Model

```mermaid
flowchart LR
    API[API contracts] --> Contracts[src/contracts]
    Config[Config contracts] --> Contracts
    Errors[Error contracts] --> Contracts
```

## Rules

- give each contract one obvious owner path
- document the promise and its intended audience
- add tests that would fail if the promise drifts
- do not hide contract truth behind convenience reexports

