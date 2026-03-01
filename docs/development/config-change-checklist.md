---
title: Config change checklist
audience: contributor
type: how-to
stability: stable
owner: platform
last_reviewed: 2026-03-01
tags:
  - configs
  - governance
related:
  - docs/reference/configs.md
  - docs/reference/schema-versioning-policy.md
  - docs/reference/config-keys-reference.md
verification:
  - cargo run -q -p bijux-dev-atlas -- contracts configs --mode static --format text
---

# Config change checklist

## Scope and ownership

- Confirm the config belongs under an existing `configs/<domain>/` directory.
- Ensure owner and consumer mappings are updated for any new or renamed file.

## Schema and registry consistency

- Update schema files only under `configs/contracts/` or `configs/schema/`.
- Update `configs/schema-map.json` when schema coverage changes.
- If a public schema changes, update `configs/schema/versioning-policy.json`.

## Deterministic outputs

- Regenerate committed config indexes through control-plane:
  - `cargo run -q -p bijux-dev-atlas -- configs list --allow-write --format text`
- Validate with contracts:
  - `cargo run -q -p bijux-dev-atlas -- contracts configs --mode static --format text`

## Drift and documentation checks

- Run: `cargo run -q -p bijux-dev-atlas -- configs verify --allow-write --strict --format text`
- Update reference docs when key names, defaults, or behavior changes.
- Do not leave placeholder values in stable configs unless marked as example-only.
