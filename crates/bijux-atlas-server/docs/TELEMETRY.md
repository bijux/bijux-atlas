# Telemetry

- Metrics use `bijux_*` naming and include stable labels: subsystem, version, dataset.
- Request metrics include route, status, latency buckets and trace exemplars when available.
- Structured JSON logs include `request_id`; trace propagation accepts `traceparent`.
- SQLite query latency and store latency/failure counters are exported via `/metrics`.
