---
title: Cluster Membership Protocol
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - membership
related:
  - docs/architecture/cluster-membership-lifecycle.md
---

# Cluster Membership Protocol

## Protocol Steps

1. Register node through `POST /debug/cluster/register`.
2. Confirm active listing via `GET /debug/cluster/nodes`.
3. Send periodic heartbeat through `POST /debug/cluster/heartbeat`.
4. Validate cluster summary through `GET /debug/cluster-status`.

## Required Fields

- `cluster_id`
- `node_id`
- `generation`
- `role`
- `capabilities`

## Acceptance Rules

1. Registration requires non-empty node identity fields.
2. Heartbeats must carry matching generation.
3. Timeout policy is enforced by membership detector.

## Policy References

- OPS-ROOT-023: operation docs that declare policy behavior must reference an OPS contract id.

## Operator Commands

- `bijux-dev-atlas system cluster membership`
- `bijux-dev-atlas system cluster node-health`
