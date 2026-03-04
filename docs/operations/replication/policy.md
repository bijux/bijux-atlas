# Replication Policy

## Core Rules

- replication factor must be at least 2 for clustered deployments.
- each replica group must have exactly one primary at any point in time.
- promotion target must already exist in replica membership.
- lag budget default is 2000 ms and should be tuned per environment.

## Operational Rules

- perform planned promotions during maintenance windows.
- do not run rebalancing and failover on the same shard concurrently.
- record every manual promotion with reason and operator identity.
