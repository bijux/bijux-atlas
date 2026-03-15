---
title: Guarantees and Stability
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Guarantees and Stability

Atlas is opinionated about stability: it does not promise everything, but what it does promise should be explicit, test-backed, and documented.

## The Stability Stack

```mermaid
flowchart TD
    Contracts[Contracts] --> Surfaces[CLI / API / Config / Errors]
    Surfaces --> Tests[Compatibility and contract tests]
    Tests --> Releases[Release confidence]
    Releases --> Users[User and operator trust]
```

Atlas aims to make stability understandable by layer:

- public commands and options are more stable than internal helper code
- API schemas and structured output are more stable than ad hoc debug payloads
- runtime config contracts are more stable than undocumented environment-dependent behavior

## What Atlas Tries to Guarantee

```mermaid
flowchart LR
    G[Guarantees] --> C1[Deterministic structured output]
    G --> C2[Stable contract-owned APIs]
    G --> C3[Explicit runtime validation]
    G --> C4[Immutable artifact-oriented workflows]
```

Atlas tries to provide:

- deterministic machine-readable output where documented
- explicit validation rather than silent coercion
- stable contract-owned API and config surfaces
- immutable artifact workflows for release state

## What Atlas Does Not Guarantee

- all internal Rust module paths remain unchanged
- all debug-only behavior remains stable
- all internal fixtures or benchmark helpers are public API
- every implementation detail remains source-compatible across refactors

## Why Stability Is Evidence-Based

```mermaid
flowchart LR
    Docs[Documentation] --> Contracts[Contract definitions]
    Contracts --> Tests[Test enforcement]
    Tests --> Evidence[Build and release evidence]
    Evidence --> Trust[Operational trust]
```

Atlas does not treat “we intended this to be stable” as enough. Stability is meaningful only when:

- the surface is documented
- ownership is clear
- tests enforce it
- releases validate it

## How to Interpret Stability in Practice

If you are a user:

- trust documented commands, config contracts, and query behavior

If you are an operator:

- trust documented runtime and operational contracts, not incidental local behavior

If you are a maintainer:

- do not turn undocumented implementation details into accidental promises

## Next Pages

- [Run Atlas Locally](../02-getting-started/run-atlas-locally.md)
- [Contracts and Boundaries](../05-architecture/contracts-and-boundaries.md)
- [Ownership and Versioning](../08-contracts/ownership-and-versioning.md)

