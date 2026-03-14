---
title: Crate API stability boundaries
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - release
  - api
related:
  - docs/reference/crate-release-policy.md
  - release/crates-v0.1.toml
---

# Crate API stability boundaries

## Stable public boundary

The following crates are public for `v0.1` and are expected to keep a stable API boundary within semver rules:

- `bijux-atlas`

## Private boundary

- `bijux-dev-atlas` is a private crate (`publish = false`).
- Their public symbols are not a consumer-facing compatibility contract.

## Internal modules

- Internal implementation modules should remain private (`mod`), and only stable entrypoints should be re-exported.
- If a module must stay public for technical reasons but should not appear in crate docs, use `#[doc(hidden)]` and document why in code review.

## Stability levels

- Stable: backward-compatible within major version.
- Experimental: may change without stability guarantee.

Every public API change must be reflected in crate release notes and semver evaluation.
