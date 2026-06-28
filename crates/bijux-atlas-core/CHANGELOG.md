# Changelog

All notable changes to **bijux-atlas-core** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Restated `bijux-atlas-core` as the runtime-independent owner for canonical
  JSON, hashing, stable sorting, and shared error-code helpers in the `0.2.2`
  release line.
- Rewrote crate-facing documentation so core reads as a stable primitive layer
  instead of a generic support package.

### Fixed
- Removed stale release documentation assumptions that still treated
  compatibility layers as core-surface owners.
- Removed ambiguity between core primitives and higher-level dataset or runtime
  behavior.
