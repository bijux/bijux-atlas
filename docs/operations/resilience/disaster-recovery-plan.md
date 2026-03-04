# Disaster Recovery Plan

## Objectives

- preserve service availability through controlled failover
- restore cluster stability with bounded recovery latency

## Activation Criteria

- multiple concurrent node failures
- sustained partition that breaks quorum routing
- repeated failed recoveries beyond retry budget

## Plan

1. freeze non-critical changes
2. promote stable replicas for critical shards
3. isolate unhealthy nodes
4. rebalance healthy ownership
5. validate diagnostics and metrics
6. restore normal routing and close incident
