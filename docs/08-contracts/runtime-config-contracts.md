---
title: Runtime Config Contracts
audience: operator
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Runtime Config Contracts

Runtime config contracts define the stable expectations around server configuration inputs and validation behavior.

## Runtime Config Contract Scope

```mermaid
flowchart LR
    RuntimeConfig[Runtime config] --> Flags[Flags]
    RuntimeConfig --> Env[Environment variables]
    RuntimeConfig --> Schema[Config schema]
```

## Contract Logic

```mermaid
flowchart TD
    Invalid[Invalid config] --> Reject[Validation should reject]
    Valid[Valid config] --> Start[Runtime may start]
```

## Main Promise

Atlas should validate explicit runtime configuration rather than silently accepting malformed or contradictory input.

