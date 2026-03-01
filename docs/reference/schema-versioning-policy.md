---
title: Schema versioning policy
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-01
tags:
  - configs
  - schema
  - versioning
related:
  - docs/reference/configs.md
  - docs/reference/config-keys-reference.md
source:
  - configs/schema/versioning-policy.json
---

# Schema versioning policy

## Canonical source

- Policy file: `configs/schema/versioning-policy.json`
- Schema map: `configs/schema-map.json`

## Policy rules

- Governed public schemas must be listed in versioning policy.
- Compatibility classification must be explicit (`backward-compatible` for current policy set).
- Versioning mode must be explicit (`locked` for current policy set).

## Required workflow for schema changes

1. Update schema file and tests.
2. Update `configs/schema-map.json` if schema coverage changes.
3. Update `configs/schema/versioning-policy.json` if governed public schemas changed.
4. Regenerate deterministic indexes via:
   - `cargo run -q -p bijux-dev-atlas -- configs list --allow-write --format text`
5. Verify:
   - `cargo run -q -p bijux-dev-atlas -- contracts configs --mode static --format text`
