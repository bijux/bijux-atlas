# Changelog

All notable changes to **bijux-atlas** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.1 – 2026-04-22

### Changed
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
