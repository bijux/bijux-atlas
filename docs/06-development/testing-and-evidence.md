---
title: Testing and Evidence
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Testing and Evidence

Atlas changes should be defended by evidence, not only by intuition.

## Evidence Model

```mermaid
flowchart LR
    Change[Code or docs change] --> Tests[Tests]
    Change --> Contracts[Contract checks]
    Change --> Docs[Docs updates]
    Tests --> Evidence[Evidence for review]
    Contracts --> Evidence
```

## Test Shape

```mermaid
flowchart TD
    Unit[Unit tests] --> Confidence[Local correctness]
    Compatibility[Compatibility tests] --> Confidence
    Interface[Interface tests] --> Confidence
    Workflow[Workflow tests] --> Confidence
```

## Practical Commands

```bash
cargo test -p bijux-atlas
cargo test -p bijux-dev-atlas
make test
```

## Maintainer Rule

If you change a public or contract-owned surface, the test story should show why the change is safe or intentionally breaking.

