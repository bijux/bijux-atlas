# Changelog

All notable changes are documented in this file.

## v0.2.0

### Added
- Added public release-facing workflow parity for CI, docs deployment, crates.io publishing, and GitHub releases.
- Added a GitHub helper surface in `makes/` so release automation can stay thin and deterministic.

### Changed
- Changed the root release story to align the README, chart version, workspace version, and release metadata around `v0.2.0`.
- Changed release specification filenames from version-shaped names to stable names under `ops/release/`.

### Fixed
- Fixed release validation to read the live docs spine for MSRV and feature-flag documentation.
- Fixed the runtime CLI config example so it satisfies the current runtime configuration contract.

### Breaking Changes
- Renamed `ops/release/crates-v0.1.toml`, `ops/release/images-v0.1.toml`, and `ops/release/ops-v0.1.toml` to stable filenames. Any automation that referenced the old paths must switch to `crates-release.toml`, `images-release.toml`, and `ops-release.toml`.

## v0.1.1

### Added
- Added installable binary entrypoints for runtime-facing crates that were previously library-only.
- Added root directory and ops subdirectory rationale documentation in `artifacts/why.md`.

### Fixed
- Fixed workspace version and internal crate pin drift by aligning the workspace to `0.1.1`.
- Fixed repo integration contract expectations for workspace package metadata version consistency.
- Fixed slow tutorial workflow summary test classification by tagging it with the `slow_` convention.
- Fixed CI workflow policy allowlist coverage for normalized temp/cache root setup steps.
- Fixed `ci-pr` supply-chain lane by routing through `bijux-dev-atlas` security commands instead of brittle tool installs.
- Fixed security supply-chain governance bootstrap by ensuring required governance evidence files are present before validation.
- Fixed system simulation and final readiness workflows by creating artifact directories before output redirection.
- Fixed dependency-review workflow behavior to avoid blocking repository validation on external platform-side issues.

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
