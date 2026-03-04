# Replication Architecture

## Scope

Replication covers shard-level data durability and availability across cluster nodes.

## Components

- `ReplicaRegistry`: authoritative runtime model for replica groups.
- `ReplicationPolicy`: factor and lag limits.
- `ConsistencyGuarantee`: read and write consistency contract.
- replication debug endpoints in server runtime.
- replication cluster commands in `bijux-dev-atlas`.

## Data Model

A replica group is identified by:

- `dataset_id`
- `shard_id`

Each group contains one primary and one or more replicas.

## Runtime Control Surface

- list: `GET /debug/cluster/replicas`
- health: `GET /debug/cluster/replicas/health`
- failover: `POST /debug/cluster/replicas/failover`
- diagnostics: `GET /debug/cluster/replicas/diagnostics`

## Operational Guarantees

- failover only promotes a node already registered as replica.
- failover updates primary ownership atomically inside registry state.
- lag and throughput are reported per replica group.
- health and failure counts are exposed through metrics and diagnostics.
