---
title: Workspace versioning policy
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - release
related:
  - docs/reference/crate-release-policy.md
  - release/crates-v0.1.toml
---

# Workspace versioning policy

## Policy

- Atlas uses a unified workspace version for the `v0.1` release line.
- Every publishable crate in `release/crates-v0.1.toml` ships with the same semantic version.
- Release tag format is `v<workspace-version>`.

## Scope

- Applies to crates listed under `publish.allow` in `release/crates-v0.1.toml`.
- Does not apply to private crates listed under `publish.deny`.

## Enforcement

- `bijux-dev-atlas release crates list` reads the v0.1 inventory.
- `bijux-dev-atlas release crates validate-metadata` verifies publishable crate metadata.
- `bijux-dev-atlas release crates validate-publish-flags` verifies `publish = false` for private crates.
