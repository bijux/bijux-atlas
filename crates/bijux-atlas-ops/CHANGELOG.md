# Changelog

All notable changes to **bijux-atlas-ops** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Added
- Established `bijux-atlas-ops` as the published Atlas operations-contract
  crate for release profiles, deployment assets, and ops-surface tests.
- Added release-facing crate guidance that explains how operators and tooling
  should consume owned stack, Kubernetes, and observability references.

### Changed
- Clarified release documentation so `bijux-atlas-ops` is described as part of
  the `0.2.2` crates.io surface while `bijux-atlas-dev` remains repository-only.
- Tightened ownership language so operations references are treated as durable
  crate contracts rather than incidental repository files.
