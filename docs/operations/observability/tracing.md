# Tracing

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide a deterministic trace-first diagnosis path.

## Trace-first diagnosis

1. Start from alert timestamp and capture a 15-minute window.
2. Identify top failing route by error count.
3. Find the slowest span group for the failing route.
4. Correlate span IDs with store and dataset operations.
5. Select mitigation from the mapped runbook.

## How to find slow queries

- filter to the failing route first
- sort traces by longest total duration
- inspect the slowest store or query span before blaming the API layer
- compare a slow trace with a healthy trace from the same route and dataset

## Required spans

- request root span
- query execution span
- store access span
- release or dataset resolution span when applicable

## Span tags and annotations

- `dataset_id`
- `query_type`
- `request_origin`
- `shard_id` when shard fanout routing is active
- error annotation on request spans for `5xx` responses

## Verify success

```bash
make ops-observability-verify
make ops-traces-check
bijux-dev-atlas observe traces verify --format json
bijux-dev-atlas observe traces coverage --format json
bijux-dev-atlas observe traces topology --format json
```

Expected result: traces can be filtered by route and correlated to failing component spans.

## Rollback

If tracing changes hide correlation between request, query, and store spans, revert the tracing configuration change and rerun trace checks.

## Next

- [Alerts](alerts.md)
- [Trace timeline examples](trace-timeline-examples.md)
- [Trace dashboard examples](trace-dashboard-examples.md)
- [Trace troubleshooting](trace-troubleshooting.md)
- [Incident Response](../incident-response.md)
