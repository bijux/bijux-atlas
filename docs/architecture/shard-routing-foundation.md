---
title: Shard Routing Foundation
audience: contributor
type: concept
stability: experimental
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - sharding
  - routing
related:
  - docs/architecture/cluster-topology.md
  - configs/contracts/cluster/shard-metadata.schema.json
---

# Shard Routing Foundation

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `experimental`
- Reason to exist: define shard model, ownership model, routing, assignment, and rebalance behavior.

## Shard Model

A shard is the unit of routing and ownership for one dataset key range.

Each shard record contains:

1. `shard_id`
2. `dataset_id`
3. `key_range_start`
4. `key_range_end`
5. `owner_node_id`
6. `replica_node_ids`

## Shard Key Strategy

Current strategy uses stable hash routing on key material with deterministic node ordering.

- Stable hash primitive: `stable_hash_u64`
- Deterministic owner selection from sorted shard identifiers

## Dataset Shard Layout

Dataset layout contract includes:

1. Declared shard count.
2. Partition hint.
3. Ownership transfer policy.

Contract schema: `configs/contracts/cluster/shard-metadata.schema.json`

## Shard Ownership Rules

Ownership rules enforce:

1. Minimum owner count is explicit.
2. Ownership transfer can be enabled or disabled by policy.
3. Transfer and relocation operations are auditable state transitions.

## Registry and Lookup

`ShardRegistry` supports:

1. Metadata storage (`upsert_shard`).
2. Lookup by shard id.
3. Lookup by dataset id.
4. Lookup by owner node id.

## Assignment Algorithm

Initial assignment uses round-robin over declared node list.

Properties:

1. Deterministic for identical input ordering.
2. Produces stable shard id layout.

## Routing Logic

Two routing paths are implemented:

1. Consistent hashing prototype (`route_by_hash`).
2. Static fallback (`route_static_fallback`) for degraded topology.

## Rebalance and Relocation

Rebalance redistributes shard ownership across active node ids.

Relocation and transfer operations:

1. `relocate_shard` changes owner intentionally.
2. `transfer_ownership` changes owner directly.
3. Both update ownership index.

## Runtime Metrics

`ShardRegistryMetrics` surfaces:

1. Shard count and healthy shard count.
2. Total load and access count.
3. Cache hit/miss counters.
4. Average latency.
