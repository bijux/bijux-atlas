# Changelog

All notable changes to **bijux-atlas-cli** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Added
- Established `bijux-atlas-cli` as the published owner of the direct
  `bijux-atlas` runtime binary and the CLI-surface tests that keep command
  dispatch aligned with Atlas runtime behavior.
- Added direct Cargo install and verification guidance for the published
  `bijux-atlas` binary.

### Changed
- Clarified release documentation so CLI ownership is expressed through the
  split crate tree rather than the historical monolithic runtime layout.
- Reframed the CLI boundary around the installed user command rather than the
  historical all-in-one runtime crate.
