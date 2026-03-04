# Replication Failover Process

## Trigger Conditions

- primary node unavailable
- repeated replica health failures on primary
- planned maintenance requiring primary drain

## Execution Steps

1. inspect current health: `system cluster replica-health`
2. choose promotion candidate from `replica_node_ids`
3. run failover command:
   `bijux-dev-atlas system cluster replica-failover --dataset-id <dataset> --shard-id <shard> --promote-node-id <node>`
4. verify ownership and health:
   `system cluster replica-list`
5. validate diagnostics:
   `system cluster replica-diagnostics`

## Validation Checklist

- promoted node becomes primary
- former primary moves to replica set
- replica health is `healthy`
- lag remains within policy budget
