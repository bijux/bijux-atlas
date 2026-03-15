---
title: Command Surface
audience: mixed
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Command Surface

This page summarizes the top-level Atlas command families.

## Top-Level Command Map

```mermaid
flowchart TD
    CLI[bijux-atlas] --> Config[config]
    CLI --> Catalog[catalog]
    CLI --> Dataset[dataset]
    CLI --> Diff[diff]
    CLI --> Gc[gc]
    CLI --> Policy[policy]
    CLI --> Ingest[ingest]
    CLI --> OpenAPI[openapi]
```

## Runtime Companions

```mermaid
flowchart LR
    AtlasCLI[bijux-atlas] --> UserWorkflows[User workflows]
    ServerCLI[bijux-atlas-server] --> Runtime[Runtime server]
    OpenAPICLI[bijux-atlas-openapi or openapi generate] --> Contracts[OpenAPI generation]
```

## Top-Level Families

- `config`: inspect config behavior
- `catalog`: validate and mutate catalog state
- `dataset`: validate, verify, publish, and pack dataset state
- `diff`: build dataset diff artifacts
- `gc`: plan and apply garbage collection
- `policy`: validate and explain active policy
- `ingest`: build validated dataset state from source inputs
- `openapi`: generate the OpenAPI description

## Related Binaries

- `bijux-atlas`
- `bijux-atlas-server`
- `bijux-atlas-openapi`

