# Crates Map

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: provide one-page crate grouping by architectural layer.

## Foundation Layer

- `bijux-atlas-core`: core invariants, canonicalization helpers, and shared primitives.
- `bijux-atlas-model`: domain model types shared across runtime and tooling.
- `bijux-atlas-policies`: policy evaluation and policy schema surfaces.

## Runtime Data Layer

- `bijux-atlas-ingest`: source ingestion, validation, and artifact build path.
- `bijux-atlas-store`: artifact and serving-store access layer.
- `bijux-atlas-query`: deterministic query execution over serving data.

## Runtime Interface Layer

- `bijux-atlas-api`: HTTP/API behavior contracts and response semantics.
- `bijux-atlas-server`: production server process, readiness, and serving controls.
- `bijux-atlas-cli`: runtime-facing CLI workflows and local operations.

## Control-Plane Layer

- `bijux-dev-atlas`: contributor and CI control-plane entrypoint.
- `ops/` surfaces: operational orchestration, validation, and reporting lanes.

## What to Read Next

- [Layering Rules](layering-rules.md)
- [Boundaries](boundaries.md)
- [Dataflow](dataflow.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
