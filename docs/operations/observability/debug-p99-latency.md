# Debug P99 Latency

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Use this path

1. Dashboard panels: store p95, sqlite p95, shed rate.
2. Trace exemplars: `artifacts/ops/observe/traces.exemplars.log`.
3. Metrics: `bijux_http_request_latency_p95_seconds`, `bijux_store_fetch_latency_p95_seconds`, `bijux_sqlite_query_latency_p95_seconds`.

## Commands

- `make ops-observability-pack-smoke`
- `make ops-observability-pack-export`
- `make observability-pack-test`
