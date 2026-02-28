# Boundaries

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@6856280c`
- Reason to exist: define enforceable crate, layer, and effects boundaries.

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
