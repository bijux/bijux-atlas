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

This page is a checked-in reference surface. Keep it synchronized with the repository state and generated evidence it summarizes.
