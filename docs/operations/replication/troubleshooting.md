# Replication Troubleshooting

## High Replication Lag

Symptoms:

- increasing `atlas_replication_lag_ms_avg`
- stale reads on replica candidates

Checks:

1. inspect health and lag in `system cluster replica-health`
2. inspect diagnostics and sync LSN values
3. verify network and storage saturation on replica nodes

Actions:

- scale query load away from replica candidates
- increase sync worker capacity
- rebalance shards if one node is overloaded

## Repeated Failover Events

Symptoms:

- rising `atlas_replica_failures_total`
- frequent ownership changes

Checks:

1. inspect failure reasons per replica
2. check node heartbeat and membership health
3. inspect shard placement on unstable nodes

Actions:

- quarantine unstable nodes
- promote stable replicas manually
- repair storage/network faults before rejoin
