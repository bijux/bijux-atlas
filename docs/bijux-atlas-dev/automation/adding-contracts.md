---
title: Adding Contracts
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Adding Contracts

Contracts turn Atlas behavior into a promise that a producer, consumer, owner,
and verifier can name. A type or JSON file becomes a governed contract only
when its authority, versioning, compatibility, validation, and release binding
are explicit.

## Contract Addition Flow

```mermaid
flowchart TD
    Need[Stable consumer need] --> Audience[Name producer and consumers]
    Audience --> Owner[Choose one authority]
    Owner --> Semantics[Define invariants and failure behavior]
    Semantics --> Version[Choose compatibility and version policy]
    Version --> Represent[Type, schema, registry, or policy representation]
    Represent --> Verify[Validator and negative fixtures]
    Verify --> Release[Bind generated forms and evidence to release identity]
```

A contract is incomplete when only valid examples pass. Negative fixtures must
prove that unknown fields, invalid identities, missing requirements, or
forbidden transitions fail at the owning boundary.

## Ownership Model

```mermaid
flowchart LR
    Authority[Owning source] --> Generated[Generated code or document]
    Authority --> Validator[Validation implementation]
    Authority --> Docs[Reader contract]
    Generated --> Parity[Parity and drift checks]
    Validator --> Evidence[Positive and negative evidence]
    Docs --> Consumers[Human and automation consumers]
    Parity --> Consumers
    Evidence --> Consumers
```

Generated Rust, JSON, OpenAPI, Markdown tables, and snapshots are projections
unless the domain explicitly assigns them authority. Mark their provenance and
regenerate them from the owner. Hand-editing two representations until they
agree creates duplicate truth and makes drift inevitable.

## Choose the Contract Form

| Form | Best suited for | Required companion |
| --- | --- | --- |
| Rust type and invariant | behavior owned inside one compiled boundary | semantic tests and stable serialization policy if exported |
| JSON Schema | external or cross-language document shape | strict validator, version field, and invalid fixtures |
| Registry | closed or governed identifiers and ownership metadata | uniqueness, completeness, and generated parity checks |
| Policy document | thresholds, allowed transitions, or release decisions | evaluator with explicit pass, fail, and invalid outcomes |
| Golden artifact | exact deterministic representation | authoritative generator and intentional review process |
| Protocol description | HTTP or plugin interaction | compatibility tests and live implementation parity |

Do not use a golden file to stand in for semantics it cannot express, or a
schema to claim values were operationally exercised. Structural validity,
semantic validity, and execution evidence are different layers.

## Evolution Model

```mermaid
stateDiagram-v2
    [*] --> Proposed
    Proposed --> Governed: owner, consumers, semantics, and verifier accepted
    Governed --> Additive: compatible optional capability added
    Governed --> Deprecated: migration channel published
    Additive --> Governed: consumers and generated forms verified
    Deprecated --> Removed: removal policy and compatibility window satisfied
    Governed --> Versioned: incompatible promise receives a new contract version
```

Adding an enum value, required field, route, default, or validation rule can be
incompatible even when parsing still succeeds. Review producer and consumer
behavior, not only schema syntax. Record how unknown versions and fields are
handled before publishing the contract.

## Acceptance Evidence

- One owning source and owner are discoverable.
- Producers and consumers are named, including generated and external users.
- Version, unknown-field, default, and compatibility behavior are explicit.
- Positive, boundary, and negative fixtures exercise semantic validation.
- Generated projections are reproducible and fail parity checks when stale.
- Machine failures use stable codes and distinguish invalid input from tool or
  dependency failure.
- Reader documentation explains the promise, enforcement point, limitations,
  and migration behavior.
- Release evidence identifies the contract and verifier versions used for the
  candidate.

Continue with [Automation Contracts](../governance/automation-contracts.md)
and [Evidence Contracts](../governance/evidence-contracts.md) for the control
plane's own promises.
