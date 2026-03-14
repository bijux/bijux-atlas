---
title: Release planning reference
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - release
related:
  - docs/reference/crates.md
  - docs/operations/release/index.md
---

# Release planning reference

## Publish target for v0.1

Publishable crates are defined in `configs/release/publish-policy.json`.

Blocked from crates.io by policy:

- `bijux-dev-atlas`

Current publishable crates:

- `bijux-atlas`

## Versioning model

- Workspace uses a unified version (`0.1.0` at this snapshot).
- Version and tag policy is defined in `configs/release/version-policy.json`.
- Changelog structure requirements are validated before release notes generation.

## Public API contracts

Each publishable crate has:

- crate `CONTRACT.md`
- crate `README.md`
- metadata fields in `Cargo.toml` (description, license, repository, documentation)

## Release operator commands

- `bijux dev atlas release plan --format text|json`
- `bijux dev atlas release validate --format text|json`
- `bijux dev atlas release tag --version <v> --tag <tag>`
- `bijux dev atlas release notes --version <v>`

## Crates.io readiness checks

- `cargo package -p <crate>`
- `cargo publish --dry-run -p <crate>`
- `cargo doc -p <crate> --no-deps`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
- `cargo test`
- `cargo nextest run --workspace --all-features`
- `cargo audit`
