# Boundaries

Owner: `architecture`  
Type: `concept`  
Reason to exist: define enforceable crate and layer boundaries for implementation and operations.

## Allowed Crate Dependencies

- `bijux-atlas-core`: no internal dependencies.
- `bijux-atlas-model` -> `bijux-atlas-core`.
- `bijux-atlas-ingest` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-store` -> `bijux-atlas-core`, `bijux-atlas-model`.
- `bijux-atlas-query` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-store`, `bijux-atlas-policies`.
- `bijux-atlas-api` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`.
- `bijux-atlas-cli` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-ingest`, `bijux-atlas-store`, `bijux-atlas-query`, `bijux-atlas-policies`.
- `bijux-atlas-server` -> `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-api`, `bijux-atlas-store`, `bijux-atlas-query`.

## Layer Boundary Rules

- `stack` provisions and tears down substrate services only.
- `k8s` owns deployment mechanics and health gates.
- `e2e` consumes canonical entrypoints and never patches infrastructure directly.
- `observe` validates telemetry and drills without deployment mutations.
- `load` runs workload and scoring, without changing cluster configuration.

## Forbidden Patterns

- Dependency edges not listed above.
- Cycles among internal crates.
- Cross-layer fixups where one layer repairs another layer's contract break.
- Query-runtime-only dependencies in pure planning modules.

## Operational Relevance

Boundary violations produce hidden coupling, nondeterministic incidents, and brittle recovery playbooks.
