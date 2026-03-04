---
title: Shard Failure Scenarios
audience: operator
type: runbook
stability: experimental
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - sharding
---

# Shard Failure Scenarios

## Scenario: Shard Unhealthy

1. Detect via `atlas_shard_healthy_total` drift.
2. Mark shard unhealthy and reroute.
3. Relocate ownership if required.

## Scenario: Routing Hot Spot

1. Detect high load/access concentration.
2. Rebalance across additional nodes.
