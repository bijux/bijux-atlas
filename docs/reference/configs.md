---
title: Configs Reference
audience: operators
type: reference
status: generated
owner: bijux-atlas-operations
last_reviewed: 2026-03-16
---

# Configs Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Stability: `stable`
- Source-of-truth: `configs/registry/inventory/consumers.json`

## Purpose

Generated reference for governed config groups and their declared consumers.

## Config Groups

| Group | Consumers |
| --- | --- |
| `examples` | `non-authoritative examples used by docs, tutorials, and validation fixtures` |
| `generated` | `bijux-dev-atlas generated config indexes and machine-written snapshots` |
| `internal` | `bijux-dev-atlas internal config support material` |
| `registry` | `config ownership and consumer registries, configs graph and explain commands` |
| `schemas` | `configs schema validation, contract output validation` |
| `sources` | `authored config inputs grouped by domain` |

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`

## Stability

Treat this page as generated evidence for governed config groups and their declared consumers.
