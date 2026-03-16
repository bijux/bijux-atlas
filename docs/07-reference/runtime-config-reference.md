---
title: Runtime Config Reference
audience: operator
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Runtime Config Reference

This page summarizes the most visible runtime configuration entrypoints for the server binary.

## Runtime Config Inputs

```mermaid
flowchart LR
    Flags[Startup flags] --> Runtime[Server runtime]
    Files[Config file] --> Runtime
    Env[Environment variables] --> Runtime
```

## Visible Server Flags

```mermaid
flowchart TD
    Server[bijux-atlas-server] --> Config[--config]
    Server --> Bind[--bind]
    Server --> Store[--store-root]
    Server --> Cache[--cache-root]
    Server --> Effective[--print-effective-config]
    Server --> Validate[--validate-config]
```

## Key Flags

- `--config`: explicit config file input
- `--bind`: network bind address
- `--store-root`: serving store root
- `--cache-root`: runtime cache root
- `--print-effective-config`: inspect resolved runtime config
- `--validate-config`: validate runtime config without normal startup

## Key Rule

`--store-root` should point at a serving store with published artifacts and catalog state, not at an ingest build root.

## Purpose

This page is the lookup reference for runtime config reference. Use it when you need the current checked-in surface quickly and without extra narrative.

## Stability

This page is a checked-in reference surface. Keep it synchronized with the repository state and generated evidence it summarizes.
