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
- `bijux-atlas-model`: persisted dataset, diff, gene, and policy types that must stay stable across runtime and tooling surfaces.
- `bijux-atlas-api`: stable API contracts, request parsing, response DTOs, and OpenAPI generation.
- `bijux-atlas`: runtime product crate that wires application flow, contracts, adapters, and orchestration around the owned model surface.
- `bijux-dev-atlas`: maintainer control-plane crate for repository governance and automation.

## Ownership Rules

- `bijux-atlas-core` must stay free of runtime transport and storage dependencies such as `axum`, `tokio`, `reqwest`, and `rusqlite`.
- `bijux-atlas-model` owns persisted dataset manifests, query value objects, and policy enums. Runtime code may re-export those types but must not redefine them.
- `bijux-atlas-api` owns API DTOs, error envelopes, and OpenAPI definitions. Runtime code may route requests through that surface but must not duplicate or redefine it.
- `bijux-dev-atlas` must not become an owner of runtime ingest/query/server behavior.
- CLI and HTTP entrypoints must call application/domain services and must not embed parsing-normalization rules inline.
- API DTO/wire shapes are owned under `src/contracts/api` and adapter HTTP DTOs, not in domain model modules.
- Bench-only logic is owned under `benches/` and test harnesses, not runtime `src/` modules.

## Dependency Direction

- `bijux-atlas-core` sits at the base of the Atlas crate graph.
- `bijux-atlas-model` may depend on `bijux-atlas-core`, but not on runtime, transport, storage, or maintainer crates.
- `bijux-atlas-api` may depend on `bijux-atlas-core` and `bijux-atlas-model`, but not on runtime, transport, storage, or maintainer crates.
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
