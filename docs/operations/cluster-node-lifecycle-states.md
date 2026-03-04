---
title: Cluster Node Lifecycle States
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - cluster
  - lifecycle
related:
  - docs/operations/cluster-membership-protocol.md
---

# Cluster Node Lifecycle States

## States

- `joining`
- `active`
- `quarantined`
- `maintenance`
- `draining`
- `recovering`
- `timed_out`
- `removed`

## Transition Rules

1. `joining` -> `active` after successful heartbeat.
2. `active` -> `timed_out` after heartbeat timeout.
3. `active` -> `maintenance` for planned work.
4. `active` -> `draining` before restart or removal.
5. `timed_out` -> `recovering` during recovery workflow.
6. `recovering` -> `active` after valid heartbeat.

## Manual State Controls

- `POST /debug/cluster/mode` with modes: `quarantine`, `maintenance`, `drain`, `restart`, `recover`, `remove`.
