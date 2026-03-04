# Recovery Workflows

## Automatic Workflow

1. detect timeout and fault events
2. run ownership failover for impacted shards
3. run replica promotion for impacted primary replicas
4. publish diagnostics and metrics

## Operator Workflow

1. inspect `/debug/recovery/diagnostics`
2. run targeted failover or chaos validation when needed
3. verify recovery counters and latency
4. close incident with recovery evidence
