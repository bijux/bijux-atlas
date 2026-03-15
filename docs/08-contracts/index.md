---
title: Contracts
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Contracts

This section describes the stable promises Atlas intentionally makes.

```mermaid
flowchart LR
    Contracts[Contracts] --> API[API compatibility]
    Contracts --> Output[Structured output]
    Contracts --> Config[Runtime config]
    Contracts --> Plugin[Plugin surface]
    Contracts --> Artifacts[Artifact and store]
    Contracts --> Ops[Operational contracts]
    Contracts --> Ownership[Ownership and versioning]
```

```mermaid
flowchart TD
    Promise[Promise] --> Docs[Documentation]
    Promise --> Tests[Test enforcement]
    Promise --> Review[Review and release decisions]
```

## Pages in This Section

- [API Compatibility](api-compatibility.md)
- [Automation Contracts](automation-contracts.md)
- [Structured Output Contracts](structured-output-contracts.md)
- [Runtime Config Contracts](runtime-config-contracts.md)
- [Plugin Contracts](plugin-contracts.md)
- [Artifact and Store Contracts](artifact-and-store-contracts.md)
- [Operational Contracts](operational-contracts.md)
- [Ownership and Versioning](ownership-and-versioning.md)
