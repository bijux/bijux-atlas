---
title: Cluster Membership Lifecycle
audience: contributor
type: concept
stability: experimental
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - cluster
  - membership
related:
  - docs/architecture/distributed-cluster-foundation.md
  - docs/architecture/cluster-topology.md
---

# Cluster Membership Lifecycle

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `experimental`
- Reason to exist: define membership protocol, heartbeat policy, timeout behavior, and lifecycle transitions for Atlas nodes.

## Membership Protocol

Membership is a contract-first workflow:

1. Node sends registration payload (`cluster_id`, `node_id`, `generation`, `role`, `capabilities`).
2. Control plane creates a membership record in `joining` state.
3. Heartbeats move node to `active` when generation matches.
4. Timeout detector marks nodes `timed_out` when heartbeats expire.

## Heartbeat Mechanism

Each node periodically sends heartbeat messages with:

1. Node identity tuple.
2. Generation number.
3. Current load percentage.
4. Emission timestamp.

Heartbeat acceptance updates node liveness and load counters.

## Heartbeat Interval Policy

`MembershipPolicy` defines:

- `heartbeat_interval_ms`
- `node_timeout_ms`

Constraint: `node_timeout_ms` must be greater than `heartbeat_interval_ms`.

## Node Timeout Detection

Timeout detector computes:

`now_unix_ms - last_heartbeat_unix_ms > node_timeout_ms`

Timed-out nodes transition to `timed_out` and are excluded from healthy active counts.

## Membership State Transitions

Supported states:

- `joining`
- `active`
- `quarantined`
- `maintenance`
- `draining`
- `recovering`
- `timed_out`
- `removed`

Operational modes (`quarantined`, `maintenance`, `draining`) are explicit state assignments and do not infer role changes.

## Registration and Join Workflow

Registration endpoint writes a membership record then activates the node for service if policy checks pass.

## Restart and Recovery Workflow

1. Restart increments node generation and sets state to `joining`.
2. Recovery sets state to `recovering`.
3. Valid heartbeat moves node back to `active`.

## Runtime Surfaces

- `POST /debug/cluster/register`
- `POST /debug/cluster/heartbeat`
- `POST /debug/cluster/mode`
- `GET /debug/cluster/nodes`
- `GET /debug/cluster-status`
