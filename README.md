# bijux-atlas

Deterministic genomics data platform and Rust-native control plane.

## Product Narrative

`bijux-atlas` provides stable data contracts, query/runtime crates, and an operational control plane designed for repeatable builds, deterministic outputs, and policy-driven governance.

## What You Get

- Atlas core crates for model, ingest, query, store, API, server, and CLI surfaces.
- Development control plane via `bijux dev atlas ...` for checks, docs, configs, and ops contracts.
- Deterministic artifact and report workflows with explicit schema versions.

## Quick Start

Start with the canonical onboarding page: `docs/START_HERE.md`

## Documentation Entrypoints

- Product docs: `docs/index.md`
- Getting started: `docs/START_HERE.md`
- Root reference map: `docs/root/INDEX.md`
- Operations docs: `docs/operations/INDEX.md`
- Contracts docs: `docs/contracts/INDEX.md`

## Repository Surfaces

- `crates/` Rust workspace crates
- `configs/` configuration and schema SSOT
- `ops/` operational manifests and contracts
- `docs/` canonical documentation tree
- `makefiles/` curated make target implementations
- `docker/` container contracts

## License

Apache License 2.0. See `LICENSE` and `docs/root/LICENSE_EXPLANATION.md`.
