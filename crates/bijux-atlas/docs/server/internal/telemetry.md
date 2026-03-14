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
  - `atlas_bulkhead_inflight{class=*}`
  - `atlas_bulkhead_saturation{class=*}`
  - `atlas_shed_total{reason=*}`
- Per-stage request micro-profiling is exported as `bijux_request_stage_latency_p95_seconds` for `dataset_open`, `query`, and `serialize`.
- Request/response size guard telemetry:
  - `bijux_http_request_size_p95_bytes`
  - `bijux_http_response_size_p95_bytes`
- Store backend fetch latency is exported as `bijux_store_fetch_latency_p95_seconds{backend=*}`.
