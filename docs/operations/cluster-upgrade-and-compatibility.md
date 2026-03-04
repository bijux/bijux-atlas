---
title: Cluster Upgrade And Compatibility
audience: operator
type: runbook
stability: experimental
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - compatibility
related:
  - docs/operations/cluster-deployment-models.md
  - docs/architecture/distributed-cluster-foundation.md
---

# Cluster Upgrade And Compatibility

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `experimental`
- Reason to exist: define durable rules for cluster upgrades and node-version compatibility.

## Upgrade Strategy

1. Validate configuration contracts in CI before rollout.
2. Upgrade control-plane binaries first.
3. Upgrade query nodes next, then ingest nodes.
4. Drain nodes before restart.
5. Verify cluster health after each wave.

## Compatibility Rules

1. `min_node_version` defines required minimum binary version.
2. `max_skew_major` bounds acceptable major-version skew.
3. Nodes outside compatibility policy must be rejected at join time.
4. Topology changes must preserve role quorum constraints.

## Rollout Guardrails

1. Keep one healthy query quorum during each upgrade step.
2. Keep one healthy ingest quorum during each upgrade step.
3. Block promotion if cluster health is `degraded` or `unavailable`.

## Evidence To Collect

1. Cluster status output before and after each step.
2. Node list with generation numbers.
3. Diagnostics snapshot from `system cluster diagnostics`.
