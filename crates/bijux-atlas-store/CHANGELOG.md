# Changelog

All notable changes to **bijux-atlas-store** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Clarified `bijux-atlas-store` as the owner of artifact layout, publication,
  backend, and manifest-lock contracts in the Atlas `0.2.2` release line.
- Expanded release-facing crate documentation so store now describes immutable
  publication semantics as its primary boundary.

### Fixed
- Updated release-facing storage references to point at crate-owned layout and
  backend modules.
- Removed ambiguity between storage ownership and runtime orchestration
  ownership.
