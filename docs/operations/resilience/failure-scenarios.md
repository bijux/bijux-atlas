# Resilience Failure Scenarios

## Node Failure

- trigger: heartbeat timeout
- expected result: node marked unreachable and recovery run moves ownership

## Shard Failure

- trigger: shard corruption event
- expected result: shard failover event recorded and diagnostics updated

## Replica Failure

- trigger: replica lag or health degradation
- expected result: replica failover and recovery event recording

## Network Partition

- trigger: partition between nodes
- expected result: partition event recorded and recovery workflow executed
