---
title: Shard Model
audience: contributor
type: concept
stability: evolving
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - sharding
related:
  - docs/architecture/shard-routing-foundation.md
---

# Shard Model

Atlas shard model defines each shard as a deterministic routing and ownership unit.

## Shard Fields

1. `shard_id`
2. `dataset_id`
3. `key_range_start`
4. `key_range_end`
5. `owner_node_id`
6. `replica_node_ids`

## Registry Role

`ShardRegistry` is the canonical runtime holder for shard metadata, health, and runtime counters.
