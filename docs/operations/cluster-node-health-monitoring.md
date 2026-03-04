---
title: Cluster Node Health Monitoring
audience: operator
type: runbook
stability: experimental
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - monitoring
related:
  - docs/operations/observability/cluster-membership-observability.md
---

# Cluster Node Health Monitoring

## Health Signals

1. `atlas_membership_active_nodes`
2. `atlas_membership_timed_out_nodes`
3. `atlas_membership_average_load_percent`
4. `cluster_node_status_report` output from `/debug/cluster/nodes`

## Health Verification Workflow

1. Check `/debug/cluster/status` for overall summary.
2. Check `/debug/cluster/nodes` for node-level state and liveness.
3. Confirm no sustained increase in timed-out node metric.
4. Confirm average node load stays within operating budget.
