# Boundaries

Owner: `architecture`  
Type: `concept`  
Reason to exist: define enforceable crate and layer boundaries.

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

## Layer Rules

- `k8s`, `e2e`, `observe`, and `load` do not patch each other’s state.
- Boundary violations are fixed in the owning layer, not by orchestration shortcuts.

## Operational Relevance

Boundary discipline preserves deterministic mitigation paths during outages.

## Related Pages

- [Architecture](index.md)
- [Effects](effects.md)
