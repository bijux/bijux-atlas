---
title: Crate release policy
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - release
related:
  - docs/reference/release-plan.md
  - docs/reference/crates.md
---

# Crate release policy

## Versioning policy

- Workspace follows unified semver versioning for v0.1 release line.
- Release tags must match `v<workspace-version>`.
- Changelog must contain `Added`, `Changed`, `Fixed`, `Breaking Changes`.
- Canonical release inventory lives in `ops/release/crates-v0.1.toml`.

## Public API contract per publishable Rust crate

Publishable crate:

- `bijux-atlas`

Each publishable Rust crate must maintain:

- `CONTRACT.md` with compatibility and breaking-change rules.
- `README.md` as crates.io-facing summary.
- Cargo metadata fields: `description`, `license`, `repository`, `homepage`, `documentation`, `readme`, `categories`, `keywords`, `edition`, `rust-version`.

## Private workspace crates

- `bijux-atlas-python`: private Python SDK bridge crate released through Python packaging workflows.
- `bijux-dev-atlas`: private control-plane crate for repository governance and automation.
