---
title: Crate Boundary Contract
audience: mixed
type: concept
status: canonical
owner: atlas-runtime
last_reviewed: 2026-05-02
---

# Crate Boundary Contract

This contract defines where Atlas code belongs and where it does not.

## Crate Map

- `bijux-atlas`: runtime product crate that wires domain, contracts, adapters, and app orchestration.
- `bijux-dev-atlas`: maintainer control-plane crate for repository governance and automation.
- `bijux-atlas-core` (introduced in iteration 01): runtime-independent domain primitives and invariants.

## Ownership Rules

- `bijux-atlas-core` must stay free of runtime transport and storage dependencies such as `axum`, `tokio`, `reqwest`, and `rusqlite`.
- `bijux-dev-atlas` must not become an owner of runtime ingest/query/server behavior.
- CLI and HTTP entrypoints must call application/domain services and must not embed parsing-normalization rules inline.
- API DTO/wire shapes are owned under `src/contracts/api` and adapter HTTP DTOs, not in domain model modules.
- Bench-only logic is owned under `benches/` and test harnesses, not runtime `src/` modules.

## Dependency Direction

- `domain` and `contracts` define stable truth.
- `app` orchestrates use-cases against domain and ports.
- `adapters` own transport and storage integrations.
- `runtime` owns process configuration and startup wiring.
- `bin/` surfaces remain thin wrappers around owned modules.

## Enforcement

Atlas enforces this contract through architecture tests in:

- `crates/bijux-atlas/tests/contracts_architecture_iteration01.rs`
- `crates/bijux-dev-atlas/tests/architecture_runtime_ownership.rs`

When those tests fail, boundary drift is treated as a product defect.
