---
title: Crate Boundary Contract
audience: mixed
type: concept
status: canonical
owner: atlas-runtime
last_reviewed: 2026-06-27
---

# Crate Boundary Contract

This contract defines where Atlas code belongs and where it does not.

## Crate Map

- `bijux-atlas-core`: runtime-independent primitives, canonical hashing, and invariants shared across Atlas crates.
- `bijux-atlas-ingest`: ingest normalization, artifact build execution, anomaly evaluation, and ingest-focused benchmarks or fixtures.
- `bijux-atlas-model`: persisted dataset, diff, gene, and policy types that must stay stable across runtime and tooling surfaces.
- `bijux-atlas-query`: query request parsing, planning, cursoring, SQLite execution, and frozen query contracts.
- `bijux-atlas-api`: stable API contracts, request parsing, response DTOs, Rust client compatibility surface, OpenAPI generation, and API-facing HTTP contract suites.
- `bijux-atlas-store`: publish-time store contracts, immutable artifact layout rules, and store backend implementations shared by runtime and tooling.
- `bijux-atlas`: runtime product crate that wires application flow, contracts, adapters, and orchestration around the owned model surface.
- `bijux-dev-atlas`: maintainer control-plane crate for repository governance and automation.

## Ownership Rules

- `bijux-atlas-core` must stay free of runtime transport and storage dependencies such as `axum`, `tokio`, `reqwest`, and `rusqlite`.
- `bijux-atlas-ingest` owns ingest normalization, anomaly thresholds, SQLite artifact generation, and ingest-focused tests or benches.
- `bijux-atlas-model` owns persisted dataset manifests, cross-crate gene or diff value objects, and policy enums. Runtime code may re-export those types but must not redefine them.
- `bijux-atlas-query` owns query request or response semantics, pagination cursors, query budgeting, SQLite query execution, and query-focused benches or fixtures.
- `bijux-atlas-api` owns API DTOs, error envelopes, OpenAPI definitions, Rust client compatibility, and API-facing HTTP contract or observability suites. Runtime code may route requests through that surface but must not duplicate or redefine it.
- `bijux-atlas-store` owns publish-time store paths, manifest-lock rules, immutable dataset publication semantics, and store-focused tests or benches.
- `bijux-dev-atlas` must not become an owner of runtime ingest/query/server behavior.
- Runtime `src/api`, `src/core`, `src/model`, and `src/query` are compatibility facades only; implementation ownership remains in the dedicated Atlas subcrates.
- Runtime `tests/interfaces/server` keeps runtime-only startup, cache, backend, and transport-wiring coverage. API-facing HTTP contract, response-shape, and observability suites belong under `crates/bijux-atlas-api/tests/`.
- CLI and HTTP entrypoints must call application/domain services and must not embed parsing-normalization rules inline.
- API DTO/wire shapes are owned under `crates/bijux-atlas-api/src/` and adapter HTTP DTOs, not in domain model modules.
- Bench-only logic is owned under `benches/` and test harnesses, not runtime `src/` modules.

## Dependency Direction

- `bijux-atlas-core` sits at the base of the Atlas crate graph.
- `bijux-atlas-ingest` may depend on `bijux-atlas-core`, `bijux-atlas-model`, and `bijux-atlas-query`, but not on runtime HTTP, maintainer crates, or deployment wiring.
- `bijux-atlas-model` may depend on `bijux-atlas-core`, but not on runtime, transport, storage, or maintainer crates.
- `bijux-atlas-query` may depend on `bijux-atlas-core` and `bijux-atlas-model`, but not on runtime transport, HTTP adapters, or maintainer crates.
- `bijux-atlas-api` may depend on `bijux-atlas-core`, `bijux-atlas-model`, and HTTP client libraries required for the published Rust client surface, but not on runtime or maintainer crates in production dependencies.
- `bijux-atlas-api` test-only dependencies may reference the runtime crate and async server harness libraries when those tests validate API compatibility against the runtime-owned router.
- `bijux-atlas-store` may depend on `bijux-atlas-core` and `bijux-atlas-model`, plus backend transport libraries required to read or publish artifacts, but not on runtime or maintainer crates.
- `domain` and `contracts` define stable truth within `bijux-atlas`.
- `app` orchestrates use-cases against domain and ports.
- `adapters` own transport and storage integrations.
- `runtime` owns process configuration and startup wiring.
- `bin/` surfaces remain thin wrappers around owned modules.

## Enforcement

Atlas enforces this contract through architecture tests in:

- `crates/bijux-atlas/tests/contracts_crate_boundary_contract.rs`
- `crates/bijux-dev-atlas/tests/architecture_runtime_ownership.rs`

When those tests fail, boundary drift is treated as a product defect.
