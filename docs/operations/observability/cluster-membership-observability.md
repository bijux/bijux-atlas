---
title: Cluster Membership Observability
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - observability
  - cluster
  - membership
related:
  - docs/operations/cluster-node-health-monitoring.md
---

# Cluster Membership Observability

## Metrics

- `atlas_membership_nodes_total`
- `atlas_membership_active_nodes`
- `atlas_membership_timed_out_nodes`
- `atlas_membership_quarantined_nodes`
- `atlas_membership_maintenance_nodes`
- `atlas_membership_draining_nodes`
- `atlas_membership_average_load_percent`

## Tracing Events

- `cluster_membership_nodes_view`
- `cluster_membership_register`
- `cluster_membership_heartbeat`
- `cluster_membership_mode_change`

## Logging Fields

- `event_id`
- `route`
- `node_id`
- `generation`
- `load_percent`
- `mode`

## Dashboard Examples

1. Membership health panel: active vs timed-out nodes.
2. Lifecycle modes panel: quarantined, maintenance, draining node counts.
3. Load panel: cluster average load with node status table.
4. Event panel: membership events grouped by `event_id`.
