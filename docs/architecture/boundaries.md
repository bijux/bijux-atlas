# Boundaries

Owner: `architecture`  
Type: `concept`  
Reason to exist: define enforceable crate and layer boundaries.

## Allowed Crate Dependencies

- `model` depends on `core`.
- `ingest` depends on `core` and `model`.
- `store` depends on `core` and `model`.
- `query` depends on `core`, `model`, `store`, and `policies`.
- `api` depends on `core`, `model`, and `query`.
- `server` depends on `api`, `query`, and `store`.

## Layer Rules

- `k8s`, `e2e`, `observe`, and `load` do not patch each other’s state.
- Boundary violations are fixed in the owning layer, not by orchestration shortcuts.

## Operational Relevance

Boundary discipline preserves deterministic mitigation paths during outages.

## Related Pages

- [Architecture](index.md)
- [Effects](effects.md)
