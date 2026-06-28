---
title: Crate Boundary Status
audience: maintainer
type: report
status: canonical
owner: atlas-runtime
last_reviewed: 2026-06-28
---

# Crate Boundary Status

This report tracks the current Atlas crate-boundary work and the remaining
separation tasks that still matter to maintainers.

## Current Focus

- clear crate map.
- forbidden runtime ownership in `bijux-atlas-dev`.
- runtime-independent core crate extraction.
- ingest boundary between entrypoints and ingest semantics.
- DTO versus domain object separation.
- CLI surface routed through app boundaries for query semantics.
- benchmark ownership under `benches/`.
- executable architecture dependency-direction tests.
- short crate-boundary contract.
- architecture maps for contributor placement decisions.

## Completion Notes

- crate boundary contract: `docs/bijux-atlas/foundations/crate-boundary-contract.md`.
- new core crate: `crates/bijux-atlas-core/`.
- ingest crate ownership: `crates/bijux-atlas-ingest/src/engine/`.
- query crate ownership: `crates/bijux-atlas-query/src/engine/`.
- DTO and domain split enforcement: `crates/bijux-atlas-api/src/dto/` and.
  `crates/bijux-atlas-server/tests/api_delivery_contracts/`
- layering contract tests:
  `crates/bijux-atlas-runtime/tests/contracts_layering_direction.rs`
- dev-atlas ownership checks: `crates/bijux-atlas-dev/tests/architecture_runtime_ownership.rs`.

## Remaining Work

- continue crate split beyond `bijux-atlas-core` into dedicated ingest/api/server/client/cli crates.
- migrate remaining CLI operations modules to app boundaries.
- move server startup orchestration into a thinner dedicated runtime host abstraction.
