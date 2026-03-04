---
title: Distributed Cluster Foundation
audience: contributor
type: concept
stability: evolving
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - distributed-systems
  - cluster
related:
  - docs/architecture/index.md
  - docs/operations/index.md
---

# Distributed Cluster Foundation

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `evolving`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the baseline distributed cluster model for Atlas runtime evolution.

## Distributed System Philosophy

Atlas treats distributed execution as a deterministic extension of single-node behavior:

1. Prefer explicit contracts over implicit runtime behavior.
2. Keep data placement and ownership inspectable.
3. Keep control-plane decisions reproducible from declared state.
4. Keep degradation predictable under partial failures.

## Cluster Topology Model

Atlas supports two topology modes:

1. `single_node`: one runtime process handles control-plane and data-plane responsibilities.
2. `clustered_static`: multiple nodes with static membership from configuration.

Dynamic discovery can be layered later without breaking contract shape.

## Node Roles

Each node has a declared role:

1. `ingest`: runs ingest and artifact materialization workloads.
2. `query`: runs serving and query execution workloads.
3. `hybrid`: runs both ingest and query workloads.

Role declaration is explicit and immutable for the node process lifetime.

## Control Plane Responsibilities

The control plane is responsible for:

1. Membership and node identity.
2. Cluster state publication.
3. Shard ownership assignment.
4. Health and readiness aggregation.
5. Upgrade compatibility checks.

## Data Plane Responsibilities

The data plane is responsible for:

1. Dataset reads and query execution.
2. Ingest execution and artifact writes.
3. Shard-local cache and routing execution.
4. Runtime telemetry emission.

## Cluster Configuration Contract

Cluster-level configuration declares:

1. Cluster identity and topology mode.
2. Discovery and bootstrap strategy.
3. Health timing policy.
4. Metadata store backend.
5. Compatibility policy.

See schema: `configs/contracts/cluster/cluster-config.schema.json`.

## Node Configuration Contract

Node-level configuration declares:

1. Node identity and role.
2. Advertised network address.
3. Capability list.
4. Readiness gates.
5. Shutdown behavior.

See schema: `configs/contracts/cluster/node-config.schema.json`.

## Node Identity Model

Node identity is the tuple:

- `cluster_id`
- `node_id`
- `generation`

`node_id` is stable across restarts. `generation` increments after restart and prevents stale ownership decisions.

## Node Address Format

Canonical address format:

`<scheme>://<host>:<port>`

Example: `http://atlas-node-1.internal:8080`

All addresses are normalized and stored exactly as declared.

## Node Capability Model

Capabilities are explicit string keys with bounded vocabulary, for example:

- `ingest.pipeline`
- `query.execute`
- `query.cache`
- `shard.rebalance`

Capability checks gate cluster assignment decisions.

## Discovery Strategy

The initial strategy is `static_seed_list`:

1. Bootstrap node reads a configured peer list.
2. Each node attempts join handshake with listed peers.
3. Membership stabilizes from successful handshakes.

## Bootstrap Mechanism

Bootstrap flow:

1. Load cluster and node configuration.
2. Validate compatibility policy.
3. Build initial membership view.
4. Publish cluster state snapshot.
5. Mark node as `ready` after readiness checks pass.

## Join and Exit Protocol

Join protocol:

1. Candidate node sends identity and capability declaration.
2. Cluster validates compatibility and role policy.
3. Cluster records membership entry.
4. Cluster broadcasts updated topology.

Exit protocol:

1. Node enters `draining` state.
2. Node relinquishes ownership claims.
3. Cluster marks node as `left`.
4. Cluster reassigns affected ownership.

## Cluster Health Model

Cluster health is aggregated from node states:

1. `healthy`: all required roles have ready nodes.
2. `degraded`: at least one required role is under minimum healthy count.
3. `unavailable`: control-plane cannot produce a stable topology.

## Node State Machine

Node state transitions:

1. `booting` -> `joining`
2. `joining` -> `ready`
3. `ready` -> `draining`
4. `draining` -> `left`
5. Any state -> `failed` on unrecoverable runtime error

Direct transitions that skip lifecycle intent are invalid.

## Node Lifecycle Stages

Lifecycle stages:

1. Bootstrap
2. Admission
3. Active service
4. Drain
5. Exit

Operational automation must keep transitions observable.

## Node Readiness Semantics

A node is ready only when:

1. Configuration is valid.
2. Membership registration is confirmed.
3. Required datasets are discoverable for the declared role.
4. Mandatory health probes pass.

## Node Shutdown Procedure

Ordered shutdown procedure:

1. Stop admitting new workload.
2. Drain in-flight workload.
3. Release ownership claims.
4. Publish final node state.
5. Exit process.

## Cluster Metadata Store

Cluster metadata store is the source of truth for:

1. Membership view.
2. Node descriptors.
3. Ownership map pointers.
4. Topology version.

Backends are configured via contract and must expose monotonic topology versions.
