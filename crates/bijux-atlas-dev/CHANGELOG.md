# Changelog

All notable changes to **bijux-atlas-dev** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Refreshed the control-plane docs-tooling dependency set and synchronized the
  pinned GitHub Actions references used by reusable CI and release lanes.
- Reworked the maintainer-facing README and handbook entry pages so repository
  governance, automation, and workflow ownership point at the live docs tree.

### Fixed
- Realigned workflow inventory records with the action pins used in CI so
  governance, supply-chain, and release checks validate the live automation
  surface instead of stale inventory data.
- Removed stale numbered handbook links from the crate README.

## v0.2.1 – 2026-04-22

### Changed
- Extended control-plane check outputs with schema-envelope metadata for
  `check-list` JSON consumers.
- Updated governance and policy law catalogs to match required suite and docs
  enforcement boundaries.

### Fixed
- Corrected docs and governance validation paths used by control-plane checks,
  including canonical handbook route expectations and required-docs coverage.
- Aligned release-evidence verification with current artifact contracts and
  active repository workflow inventory.
- Normalized control-plane CI behavior for local reusable workflow refs, action
  pin checks, rust toolchain pinning, and docs-health readiness execution.

## v0.2.0

### Added
- Initial workspace control-plane release line for `bijux-atlas-dev`.
