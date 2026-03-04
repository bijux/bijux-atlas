---
title: Cluster Node Maintenance Procedures
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - maintenance
related:
  - docs/operations/cluster-node-lifecycle-states.md
---

# Cluster Node Maintenance Procedures

## Planned Maintenance

1. Set node mode to maintenance.
2. Verify state in `/debug/cluster/nodes`.
3. Perform maintenance actions.
4. Move node to recovering mode.
5. Confirm node returns to active state after heartbeat.

## Planned Drain

1. Set node mode to drain.
2. Wait for in-flight work completion.
3. Restart or remove node.
4. Re-register node and confirm active status.

## Commands

- `bijux-dev-atlas system cluster node-maintenance --node-id <id>`
- `bijux-dev-atlas system cluster node-drain --node-id <id>`
