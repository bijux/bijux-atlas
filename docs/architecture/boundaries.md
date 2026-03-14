# Boundaries

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define enforceable crate, layer, and effects boundaries in plain language.

## Plain-language summary

Atlas keeps runtime behavior predictable by separating the governed runtime crate, the Python SDK distribution crate, and the control plane. Each layer has clear allowed dependencies and explicit effects ownership.

## Allowed Crate Dependencies

- `bijux-atlas` -> workspace-external crates only.
- `bijux-atlas-python` -> workspace-external crates only.
- `bijux-dev-atlas` -> `bijux-atlas`.

## Effects Model

- Filesystem effects are owned by ingest, store, and control-plane surfaces.
- Network effects are owned by server/API runtime and explicitly gated ops tooling.
- Subprocess effects are owned by control-plane ops surfaces, never hidden in runtime crates.

## Layer Rules

- `k8s`, `e2e`, `observe`, and `load` do not patch each other state.
- Boundary violations are fixed in the owning layer, not by orchestration shortcuts.
- Contract checks enforce dependency and behavior boundaries.

## Layering rules (merged)

- The runtime crate owns user-facing execution and embedded runtime modules.
- The Python SDK crate owns Python distribution concerns and does not become an alternative runtime authority surface.
- The control-plane crate orchestrates checks and operations and does not become runtime business logic.

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
