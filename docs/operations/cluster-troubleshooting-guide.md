---
title: Cluster Troubleshooting Guide
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - troubleshooting
related:
  - docs/operations/cluster-node-health-monitoring.md
  - docs/operations/observability/cluster-membership-observability.md
---

# Cluster Troubleshooting Guide

## Symptom: Node Timed Out

1. Inspect `atlas_membership_timed_out_nodes` in metrics.
2. Inspect `/debug/cluster/nodes` for last heartbeat timestamp.
3. Check heartbeat sender path health.
4. Set node to recovering mode and resend heartbeat.

## Symptom: Membership Not Active After Registration

1. Verify registration payload has correct `generation`.
2. Verify heartbeat payload matches node generation.
3. Check membership event logs for mode changes.

## Symptom: High Cluster Load

1. Inspect `atlas_membership_average_load_percent`.
2. Inspect node-level load percentages.
3. Drain overloaded node if needed.
4. Rebalance workload across healthy nodes.
