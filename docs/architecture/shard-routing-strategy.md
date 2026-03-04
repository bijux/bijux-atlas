---
title: Shard Routing Strategy
audience: contributor
type: concept
stability: experimental
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - routing
related:
  - docs/architecture/shard-model.md
---

# Shard Routing Strategy

## Primary Path

- Deterministic hash routing using `stable_hash_u64`.

## Fallback Path

- Static fallback to first available shard when routing context is degraded.

## Rebalance

- Rebalance distributes ownership across active nodes using deterministic ordering.
