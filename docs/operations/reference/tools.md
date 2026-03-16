---
title: Tools Reference
audience: operators
type: reference
status: generated
owner: bijux-atlas-operations
last_reviewed: 2026-03-16
---

# Tools Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Stability: `stable`
- Source-of-truth: `ops/inventory/tools.toml`

## Purpose

Generated reference for governed external tools and probe contracts.

## Tools

| Tool | Required | Probe Args | Version Regex |
| --- | --- | --- | --- |
| `helm` | `true` | `version --short` | `(\d+\.\d+\.\d+)` |
| `k6` | `false` | `version` | `(\d+\.\d+\.\d+)` |
| `kind` | `true` | `--version` | `(\d+\.\d+\.\d+)` |
| `kubectl` | `true` | `version --client --short` | `(\d+\.\d+\.\d+)` |

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`

## Stability

This page reflects the checked-in tool inventory and should remain generated rather than edited by hand.
