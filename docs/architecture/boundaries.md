# Boundaries

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@bdd91bc0`
- Reason to exist: define enforceable crate, layer, and effects boundaries in plain language.

## Plain-language summary

Atlas keeps runtime behavior predictable by separating data ingestion, storage, querying, and serving responsibilities. Each layer has clear allowed dependencies and explicit effects ownership.

## Allowed Crate Dependencies

- `bijux-atlas-core` -> (none).
- `bijux-atlas-model` -> (none).
- `bijux-atlas-policies` -> (none).
- `bijux-atlas-ingest` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-store` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-query` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-store`.
- `bijux-atlas-api` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`.
- `bijux-atlas-cli` -> `bijux-atlas-core`, `bijux-atlas-ingest`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-query`, `bijux-atlas-store`.
- `bijux-atlas-server` -> `bijux-atlas-api`, `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`, `bijux-atlas-store`.

## Effects Model

- Filesystem effects are owned by ingest, store, and control-plane surfaces.
- Network effects are owned by server/API runtime and explicitly gated ops tooling.
- Subprocess effects are owned by control-plane ops surfaces, never hidden in runtime crates.

## Layer Rules

- `k8s`, `e2e`, `observe`, and `load` do not patch each other state.
- Boundary violations are fixed in the owning layer, not by orchestration shortcuts.
- Contract checks enforce dependency and behavior boundaries.

## Layering rules (merged)

- Foundation crates do not depend on runtime interface crates.
- Runtime data crates can depend on foundation crates and approved adjacent data crates.
- Runtime interface crates expose behavior but do not mutate ingest artifacts directly.
- Control-plane surfaces orchestrate checks and operations and do not become runtime business logic.

## Contract-to-runtime mapping (merged)

- Ingest behavior is constrained by ingest validation contracts.
- Artifact publication is constrained by artifact schema contracts.
- Registry progression is constrained by registry and compatibility contracts.
- Query and API serving are constrained by query/API contracts and error taxonomy.

## Contract, Observability, and Security Boundaries

- Contracts: registry and runtime contracts gate allowed surface changes.
- Observability: metrics and tracing boundaries are explicit between runtime and control-plane.
- Security: privileged operations are isolated in controlled operator or CI lanes.

## Operational Relevance

Boundary discipline preserves deterministic mitigation paths during outages.

## What This Page Is Not

This page is not a contributor onboarding checklist and not a deployment guide.

## Example

```text
If query behavior changes, update query-layer contracts; do not patch API handlers as a shortcut.
```

## Terminology used here

- Boundary: [Glossary](../glossary.md)
- Contract: [Glossary](../glossary.md)
- Lane: [Glossary](../glossary.md)

## What to Read Next

- [Architecture](index.md)
- [Effects](effects.md)
- [Dataflow](dataflow.md)
- [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
