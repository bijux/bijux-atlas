# Dashboard Data Sources

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Primary Data Sources

- Prometheus: runtime, ingest, query, and registry metrics.
- Trace backend: request flow latency and dependency spans.
- Log backend: structured events and classified errors.

## Required Source Integrity

- Source endpoints must be versioned and monitored.
- Metric names must conform to metrics naming policy.
- Query expressions in dashboard JSON must remain deterministic.
