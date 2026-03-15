---
title: Ownership and Versioning
audience: maintainer
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Ownership and Versioning

Ownership and versioning contracts explain how Atlas keeps stable promises tied to one obvious owner path and evolves them intentionally.

## Ownership Model

```mermaid
flowchart LR
    Contracts[src/contracts] --> Owners[Single contract owner path]
    Owners --> Tests[Tests]
    Owners --> Docs[Docs]
```

## Versioning Logic

```mermaid
flowchart TD
    Change[Surface change] --> Compatible[Compatible evolution]
    Change --> Breaking[Breaking evolution]
    Compatible --> Versioning[Versioning decision]
    Breaking --> Versioning
```

## Main Promise

Atlas should not hide stable truth behind multiple competing roots. If a contract is real, it should have one obvious owner and an intentional versioning story.

