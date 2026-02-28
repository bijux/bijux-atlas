# Layering Rules

- Owner: `architecture`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: define allowed dependency directions in human-readable form.

## Dependency Direction

- Foundation layer has no runtime dependency on higher layers.
- Runtime data layer depends only on foundation and adjacent data-layer crates.
- Runtime interface layer depends on foundation and runtime data/query crates.
- Control-plane layer orchestrates checks and operations, but does not become runtime business logic.

## Allowed Patterns

- Query reads immutable artifacts through store-defined contracts.
- API exposes query behavior without mutating persisted release state.
- Control-plane invokes runtime or ops surfaces through explicit command contracts.

## Forbidden Patterns

- Interface layer writing directly to ingest artifacts.
- Runtime crates invoking control-plane-only orchestration helpers.
- Undocumented cross-layer shortcuts that bypass contracts.

## What to Read Next

- [Boundaries](boundaries.md)
- [Crates Map](crates-map.md)
- [Development Control Plane](../development/control-plane.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `policy`
- Stability: `stable`
- Owner: `architecture`
