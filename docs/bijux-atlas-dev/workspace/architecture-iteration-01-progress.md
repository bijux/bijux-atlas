---
title: Architecture Iteration 01 Progress
audience: maintainer
type: report
status: canonical
owner: atlas-runtime
last_reviewed: 2026-05-02
---

# Architecture Iteration 01 Progress

This report tracks the first architecture-focused execution slice against the Atlas backlog.

## Selected Goals

1. Goal 1: clear crate map
2. Goal 2: forbidden runtime ownership in `bijux-dev-atlas`
3. Goal 3: runtime-independent core crate extraction
4. Goal 4: ingest boundary between entrypoints and ingest semantics
5. Goal 5: wire DTO vs domain object separation
6. Goal 7: CLI surface routed through app boundary for query semantics
7. Goal 8: benchmark ownership under `benches/`
8. Goal 9: executable architecture dependency-direction tests
9. Goal 10: short crate-boundary contract
10. Goal 99: architecture maps for contributor placement decisions

## Completion Notes

- crate boundary contract: `docs/bijux-atlas/foundations/crate-boundary-contract.md`
- new core crate: `crates/bijux-atlas-core/`
- ingest facade: `crates/bijux-atlas/src/app/ingest/mod.rs`
- query facade: `crates/bijux-atlas/src/app/query/mod.rs`
- DTO/domain split enforcement: `crates/bijux-atlas/tests/contracts_wire_domain_separation.rs`
- layering contract tests: `crates/bijux-atlas/tests/contracts_layering_direction.rs`
- dev-atlas ownership checks: `crates/bijux-dev-atlas/tests/architecture_runtime_ownership.rs`

## Remaining Work

- continue crate split beyond `bijux-atlas-core` into dedicated ingest/api/server/client/cli crates
- migrate remaining CLI operations modules to app boundaries
- move server startup orchestration into a thinner dedicated runtime host abstraction
