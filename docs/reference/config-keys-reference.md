---
title: Config keys reference
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-01
tags:
  - configs
  - keys
related:
  - docs/reference/configs.md
  - docs/reference/schema-versioning-policy.md
source:
  - configs/schema-map.json
---

# Config keys reference

This page is a deterministic projection of schema coverage from `configs/schema-map.json`.

## Canonical key coverage map

| Config pattern | Schema |
| --- | --- |
| `configs/_generated/configs-index.json` | `configs/schema/configs-index.schema.json` |
| `configs/ci/env-contract.json` | `configs/contracts/env.schema.json` |
| `configs/ci/lanes.json` | `configs/contracts/inventory-commands.schema.json` |
| `configs/configs.contracts.json` | `configs/schema/configs-contracts.schema.json` |
| `configs/consumers-registry.json` | `configs/schema/configs-consumer-map.schema.json` |
| `configs/docs/*.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/docs/*.jsonc` | `configs/contracts/inventory-configs.schema.json` |
| `configs/docs/.markdownlint-cli2.jsonc` | `configs/contracts/inventory-configs.schema.json` |
| `configs/gates/*.json` | `configs/contracts/inventory-commands.schema.json` |
| `configs/inventory.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/inventory/configs.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/inventory/consumers.json` | `configs/schema/configs-consumer-map.schema.json` |
| `configs/inventory/groups.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/inventory/owners.json` | `configs/contracts/inventory-owners.schema.json` |
| `configs/layout/*.json` | `configs/contracts/inventory-budgets.schema.json` |
| `configs/make/*.json` | `configs/contracts/inventory-make.schema.json` |
| `configs/meta/*.json` | `configs/schema/meta-ownership.schema.json` |
| `configs/openapi/**/*.json` | `configs/schema/public-surface.schema.json` |
| `configs/ops/*.json` | `configs/contracts/inventory-ops.schema.json` |
| `configs/ops/*/*.json` | `configs/contracts/inventory-ops.schema.json` |
| `configs/owners-registry.json` | `configs/schema/configs-owner-map.schema.json` |
| `configs/owners/*.json` | `configs/contracts/inventory-owners.schema.json` |
| `configs/perf/*.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/policy/*.json` | `configs/policy/policy.schema.json` |
| `configs/product/*.json` | `configs/product/artifact-manifest.schema.json` |
| `configs/repo/*.json` | `configs/schema/public-surface.schema.json` |
| `configs/rust/*.json` | `configs/contracts/inventory-configs.schema.json` |
| `configs/schema-map.json` | `configs/schema/configs-schema-map.schema.json` |
| `configs/schema/generated/*.json` | `configs/schema/configs-schema-index.schema.json` |
| `configs/schema/versioning-policy.json` | `configs/schema/versioning-policy.schema.json` |
| `configs/slo/*.json` | `configs/contracts/inventory-configs.schema.json` |

## Regeneration

Regenerate this mapping source through control-plane:

- `cargo run -q -p bijux-dev-atlas -- configs list --allow-write --format text`
