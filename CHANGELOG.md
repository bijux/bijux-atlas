# Changelog
<a id="top"></a>

All notable changes to **Bijux Atlas** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

---

## Unreleased

### Changed
- Refreshed dependency lockfiles to incorporate the current patched Rust and docs-tooling package versions tracked by security advisories.
- Added explicit docs-tooling transitive dependency overrides to keep patched `flatted` and `picomatch` versions stable across installs.

### Fixed
- Resolved all open Atlas Dependabot alerts and cleared superseded dependency-update pull requests.

## v0.2.0

### Added
- Added the public runtime release line around the shipped binaries `bijux-atlas`, `bijux-atlas-server`, and `bijux-atlas-openapi`, plus the workspace maintainer control plane `bijux-dev-atlas`.
- Added a numbered documentation spine and GitHub Pages structure that map directly to the live runtime, operations, contracts, development, and reference surfaces.
- Added release evidence generation, signing, packet assembly, and verification flows that bundle docs, SBOMs, ops reports, and publish artifacts under `ops/release/`.
- Added crates.io release support for `bijux-atlas` with crate-owned runtime contracts, packaged security policy inputs, and publish-time validation that succeeds from the crate tarball instead of only from the workspace checkout.
- Added public GitHub release workflows and a thin `makes/` helper surface so automation can stay declarative while the real orchestration lives in Rust.

### Changed
- Changed the repository layout to use durable ownership boundaries across `crates/`, `docs/`, `ops/`, `configs/`, `.github/`, and root metadata so the code, contracts, and operating guidance describe the same system.
- Changed runtime and release version identity to derive from real `v*` git tags, keeping checkout builds honest about the latest published release line until a newer tag exists.
- Changed authoritative repository inputs to live under `configs/sources/...`, with generated material kept separate from source authority data and checked through the maintainer control plane.
- Changed the Atlas README, crate READMEs, docs navigation, contributor guidance, security policy, and workflow templates to match the current product and maintainer surfaces instead of older split-layout assumptions.
- Changed release specification filenames from version-shaped names to stable names under `ops/release/` so automation refers to durable contracts rather than release-era placeholders.

### Removed
- Removed split legacy crate trees, duplicate root-level runtime wrappers, stale compatibility facades, and placeholder module paths that no longer matched the merged `bijux-atlas` runtime ownership model.
- Removed obsolete docs, generated registry authority drift, stale config aliases, and retired ops markdown surfaces that were creating duplicate or misleading sources of truth.
- Removed release and governance assumptions that depended on manual interpretation instead of validated repository contracts and signed evidence outputs.

### Fixed
- Fixed the ingest SQLite golden hash checks so cross-machine determinism no longer depends on volatile build metadata.
- Fixed crates.io packaging by embedding runtime contract assets inside the published crate instead of reading files from workspace-only paths.
- Fixed release validation to read dependency, governance, compatibility, and docs policy from canonical sources, including publishable-crate filtering for dependency policy checks.
- Fixed security workflow pin validation so GitHub Actions pinned by SHA remain valid even when inline version comments are present.
- Fixed release evidence signing and verification so the normalized bundle includes the same ops evidence and report artifacts declared in the manifest, and `release verify --evidence` accepts the evidence directory contract directly.
- Fixed docs redirects, deprecation validation, compatibility rule coverage, and stale operations/security references so repository guidance stays aligned with validated live surfaces.

### Breaking Changes
- Renamed `ops/release/crates-v0.1.toml`, `ops/release/images-v0.1.toml`, and `ops/release/ops-v0.1.toml` to `crates-release.toml`, `images-release.toml`, and `ops-release.toml`.
- Renamed the public runtime binaries to the `bijux-atlas*` family and moved the stable umbrella namespaces to `bijux atlas ...` and `bijux dev atlas ...`.

[Back to top](#top)

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
