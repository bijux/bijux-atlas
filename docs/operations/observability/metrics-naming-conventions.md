# Metrics Naming Conventions

- Canonical namespace: `atlas_*`.
- Compatibility namespace allowed only for migration: `bijux_*`.
- Counter suffix: `_total`.
- Latency suffix: `_seconds`.
- Size suffix: `_bytes`.
- Count histogram suffix: `_count`.

Examples:

- `atlas_http_requests_total`
- `atlas_http_request_duration_seconds`
- `atlas_query_rows_count`
