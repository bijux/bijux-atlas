# What Breaks If Removed Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`

## Removal Impact Targets

| Path | Impact | Consumers |
| --- | --- | --- |
| `ops/inventory/contracts-map.json` | `breaking` | `checks_ops_inventory_contract_integrity, checks_ops_file_usage_and_orphan_contract` |
| `ops/load/scenarios` | `warning` | `checks_ops_minimalism_and_deletion_safety` |

## Purpose

This page is the lookup reference for what breaks if removed reference. Use it when you need the current checked-in surface quickly and without extra narrative.

## Stability

This page is a checked-in reference surface. Keep it synchronized with the repository state and generated evidence it summarizes.
