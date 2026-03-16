---
title: What Breaks If Removed Reference
audience: operators
type: reference
status: generated
owner: bijux-atlas-operations
last_reviewed: 2026-03-16
---

# What Breaks If Removed Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Stability: `stable`
- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`

## Purpose

Generated reference for curated removal-impact evidence.

## Removal Impact Targets

| Path | Impact | Consumers |
| --- | --- | --- |
| `ops/inventory/contracts-map.json` | `breaking` | `checks_ops_inventory_contract_integrity, checks_ops_file_usage_and_orphan_contract` |
| `ops/load/scenarios` | `warning` | `checks_ops_minimalism_and_deletion_safety` |

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`

## Stability

This page should reflect the checked-in removal-impact example report and stay generated from that input.
