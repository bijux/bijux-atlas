# Replication Model

## Purpose

Replication keeps shard data available during node outages while preserving predictable read and write behavior.

## Replication Policy

- `replication_factor`: default `2` (one primary + one replica)
- `primary_required`: `true`
- `max_replication_lag_ms`: `2000`

## Primary And Replica Roles

- The primary replica receives writes and advances the source log sequence number.
- Secondary replicas apply source changes in order and expose health + lag telemetry.
- Failover promotes an existing replica to primary and demotes the previous primary.

## Synchronization Strategy

Each replica group tracks:

- `primary_lsn`
- `last_applied_lsn`
- `lag_ms`
- `sync_throughput_rows_per_second`

Synchronization is pull-based from the promoted primary and is measured per shard group.

## Consistency Guarantees

Default consistency contract:

- read consistency: `quorum`
- write consistency: `quorum`

This default balances availability and correctness for clustered deployments.

## Runtime Interfaces

Server debug endpoints:

- `GET /debug/cluster/replicas`
- `GET /debug/cluster/replicas/health`
- `POST /debug/cluster/replicas/failover`
- `GET /debug/cluster/replicas/diagnostics`

CLI commands:

- `bijux-dev-atlas system cluster replica-list`
- `bijux-dev-atlas system cluster replica-health`
- `bijux-dev-atlas system cluster replica-failover`
- `bijux-dev-atlas system cluster replica-diagnostics`

## Metrics

- `atlas_replica_groups_total`
- `atlas_replica_healthy_groups_total`
- `atlas_replication_lag_ms_avg`
- `atlas_replication_throughput_rows_per_second`
- `atlas_replica_failures_total`
