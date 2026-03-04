---
title: Crate dependency policy
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - crates
  - dependencies
related:
  - docs/reference/crates.md
  - release/crates-v0.1.toml
---

# Crate dependency policy

## Policy

- Publishable crates must avoid git dependencies.
- Publishable crates must avoid direct path dependencies in package manifests.
- Default features should stay minimal and intentional.
- Dependency additions should be justified by runtime value and maintenance cost.

## Verification

- `bijux-dev-atlas release validate`
- `bijux-dev-atlas release crates dry-run`

## Audit output

Dependency audits are emitted as release reports so reviewers can track dependency growth and risk classes over time.
