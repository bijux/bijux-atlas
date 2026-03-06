# Changelog

All notable changes are documented in this file.

## v0.1.0

### Added
- Established a Rust workspace with runtime crates and a dedicated control-plane crate.
- Delivered runtime crates for core, model, policies, store, query, ingest, api, client, cli, and server.
- Standardized command surfaces so make/workflows route through `bijux-dev-atlas`.
- Built contract-driven governance across `ops/`, `configs/`, `docs/`, `make/`, and root surfaces.
- Added docs inventory, link validation, nav integrity checks, and generated docs registries.
- Introduced real-data tutorial run contracts, dataset metadata, and evidence-oriented docs pages.
- Added release and compatibility reporting surfaces with machine-readable JSON outputs.
- Added GitHub Actions lanes for release candidate, docs deploy, ops validate/publish, and crates publish.
- Added container, helm, and ops packaging workflows aligned with deterministic artifact generation.
- Set Apache-2.0 licensing and release metadata baseline for `v0.1.0`.
