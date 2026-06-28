# Changelog

All notable changes to **bijux-atlas-cli** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Added
- Established `bijux-atlas-cli` as the published owner of the direct
  `bijux-atlas` runtime binary and the CLI-surface tests that keep command
  dispatch aligned with Atlas runtime behavior.
- Added release-facing install and verification guidance for the direct Cargo
  binary path.

### Changed
- Clarified release documentation so CLI ownership is expressed through the
  split crate tree rather than the historical monolithic runtime layout.
- Restated the CLI boundary so end-user command ownership is separate from
  server, OpenAPI, and runtime orchestration ownership.
