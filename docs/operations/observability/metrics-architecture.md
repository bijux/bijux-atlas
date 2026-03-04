# Metrics Architecture

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define metrics philosophy, naming, labels, namespace, and cardinality rules.

## Observability philosophy

Atlas metrics are designed for operational decisions, not vanity reporting.
Every metric must answer one of these questions: is the service healthy, where is latency spent, where are errors concentrated, or is capacity saturating.

## Naming conventions

- Primary runtime namespace: `atlas_*`
- Compatibility namespace retained where needed: `bijux_*`
- Counters end with `_total`
- Histograms end with `_seconds`, `_bytes`, or `_count` based on unit
- Gauges use descriptive unit suffixes (for example `_bytes`, `_ratio`)

## Namespace policy

The canonical namespace is `atlas_*`. Compatibility aliases may coexist during migration windows.

## Label conventions

Required baseline labels:

- `subsystem`
- `version`
- `dataset`

Route-level request metrics may also include:

- `route`
- `method`
- `status`
- `class`

## Cardinality policy

- Disallow raw user identifiers in metric labels.
- Use bounded enums for `class`, `query_type`, and `backend` labels.
- Hash or bucket any dataset-derived label with unbounded raw values.
- Add new labels only with explicit operational justification.

## Runtime implementation

- Metrics module: `crates/bijux-atlas-server/src/telemetry/metrics.rs`
- Metrics endpoint: `GET /metrics`
- Initialization path: `AppState::init_request_metrics()`
- Endpoint toggle: `ATLAS_ENABLE_METRICS_ENDPOINT`

## Required request and query metrics

- request count: `atlas_http_requests_total`
- request latency histogram: `atlas_http_request_duration_seconds`
- request error count: `atlas_http_request_errors_total`
- request and response size metrics
- query execution latency histogram
- query plan generation latency histogram
- query row count histogram
- query cache hits and misses

## Required runtime capacity metrics

- ingest throughput and ingest pipeline stage latency
- dataset load duration
- shard load, evictions, hit rate, and miss rate
- cache memory usage and entry count
- process memory, CPU usage ratio, and open file descriptors
- thread pool usage ratio
- runtime queue depth and task backlog
- slow query counter
- dataset query distribution
