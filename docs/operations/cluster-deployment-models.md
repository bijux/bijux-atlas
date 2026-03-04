---
title: Cluster Deployment Models
audience: operator
type: runbook
stability: experimental
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - deployment
related:
  - docs/architecture/cluster-topology.md
  - docs/operations/upgrade-compatibility-guide.md
---

# Cluster Deployment Models

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `experimental`
- Reason to exist: define single-node and multi-node deployment models and their operational tradeoffs.

## Single-Node Mode

Use `single_node` when environment simplicity is more important than horizontal scaling.

1. One node runs control-plane and data-plane workloads.
2. Recommended for local development and contract verification.
3. Recovery is process restart oriented.

## Multi-Node Mode

Use `clustered_static` when ingest and query isolation or horizontal query capacity is required.

1. Control-plane decisions are shared across multiple nodes.
2. Ingest and query roles can be split by node capability.
3. Requires explicit seed-list and metadata store configuration.

## Mode Selection Guidance

1. Start in single-node for deterministic baseline.
2. Move to multi-node when workload isolation is needed.
3. Keep contract files versioned and reviewed before rollout.

## Validation Checklist

1. Validate `cluster-config` and `node-config` against schemas.
2. Confirm `/debug/cluster-status` returns `healthy` before promotion.
3. Record topology snapshot artifact for audit trails.
