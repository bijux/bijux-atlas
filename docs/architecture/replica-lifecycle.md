# Replica Lifecycle

## States

- provisioning
- synchronizing
- healthy
- degraded
- promoted
- recovering

## Transitions

1. provisioning -> synchronizing
2. synchronizing -> healthy
3. healthy -> degraded (health failure)
4. degraded -> promoted (failover)
5. promoted -> recovering (old primary rejoins)
6. recovering -> healthy

## Key Signals

- `lag_ms`
- `sync_throughput_rows_per_second`
- `failed_checks`
- `last_failure_reason`

## Promotion Rules

A replica can be promoted when:

- it is listed in `replica_node_ids`
- it has recent sync state
- operator or automation triggers failover
