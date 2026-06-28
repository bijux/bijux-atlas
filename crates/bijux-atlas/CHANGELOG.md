# Changelog

All notable changes to **bijux-atlas** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Updated the runtime telemetry stack to `opentelemetry` `0.32`,
  `opentelemetry-otlp` `0.32`, `opentelemetry_sdk` `0.32.1`, and
  `tracing-opentelemetry` `0.33` so Atlas runtime builds and publish-time
  packaging resolve against the current observability baseline.
- Reworked compatibility-crate release notes and README guidance so the alias
  contract reads as a real Rust import surface rather than decorative migration
  text.

### Fixed
- Refreshed the release lockfile inputs that back telemetry-related transitive
  dependencies so the published crate builds with the hardened dependency set
  prepared in this release line.
- Removed an unpublished compatibility-crate entry so the crate-local release
  history matches the actual published Atlas tags.
