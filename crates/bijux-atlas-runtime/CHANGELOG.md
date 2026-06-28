# Changelog

All notable changes to **bijux-atlas-runtime** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Rebased the release-facing runtime documentation on the split crate layout,
  with `bijux-atlas-runtime` kept as orchestration, `bijux-atlas` kept as the
  compatibility alias, and CLI, server, or API owners documented as separate
  crates.
- Clarified direct Cargo-installed binary routes and the distinction between
  runtime composition ownership and installed binary ownership.

### Fixed
- Removed stale package and source-layout references that still described the
  old pre-split `crates/bijux-atlas/src/...` runtime tree as the canonical
  implementation path.

## v0.2.1 – 2026-04-22

### Changed
- Renamed the canonical runtime crate from `bijux-atlas` to
  `bijux-atlas-runtime` so the shorter `bijux-atlas` package can exist as a
  compatibility alias without owning the implementation.
- Tightened runtime feature wiring so `backend-local` enables serde-backed
  canonicalization paths consistently across crate builds.
- Applied server-surface cleanup to satisfy strict clippy requirements without
  relaxing workspace lint contracts.

### Fixed
- Aligned runtime-facing docs validation assumptions with the current Atlas
  documentation taxonomy used by repository checks.

## v0.2.0

### Added
- Initial public release line for the Atlas runtime crate, including
  `bijux-atlas`, `bijux-atlas-server`, and `bijux-atlas-openapi`.
