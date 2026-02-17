# Telemetry

- Metrics use `bijux_*` naming and include stable labels: subsystem, version, dataset.
- Request metrics include route, status, latency buckets and trace exemplars when available.
- Structured JSON logs include `request_id`; trace propagation accepts `traceparent`.
- SQLite query latency and store latency/failure counters are exported via `/metrics`.
- Overload and resilience metrics include:
  - `bijux_request_queue_depth`
  - `bijux_disk_io_latency_p95_ns`
  - `bijux_fs_space_pressure_events_total`
  - `bijux_overload_shedding_active`
- Per-stage request micro-profiling is exported as `bijux_request_stage_latency_p95_seconds` for `dataset_open`, `query`, and `serialize`.
