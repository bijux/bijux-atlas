# Observability Surface

Generated from observability contract SSOT files:
- `ops/observability/contract/metrics-contract.json`
- `ops/observability/contract/alerts-contract.json`
- `ops/observability/contract/dashboard-panels-contract.json`
- `ops/observability/contract/logs-fields-contract.json`

## Metrics
- `atlas_bulkhead_inflight`
- `atlas_bulkhead_saturation`
- `atlas_overload_active`
- `atlas_policy_relaxation_active`
- `atlas_policy_violations_total`
- `atlas_shed_total`
- `bijux_dataset_count`
- `bijux_dataset_disk_usage_bytes`
- `bijux_dataset_hits`
- `bijux_dataset_misses`
- `bijux_disk_io_latency_p95_ns`
- `bijux_errors_total`
- `bijux_fs_space_pressure_events_total`
- `bijux_http_request_latency_p95_seconds`
- `bijux_http_request_size_p95_bytes`
- `bijux_http_requests_total`
- `bijux_http_response_size_p95_bytes`
- `bijux_overload_shedding_active`
- `bijux_request_queue_depth`
- `bijux_request_stage_latency_p95_seconds`
- `bijux_runtime_policy_hash`
- `bijux_sqlite_query_latency_p95_seconds`
- `bijux_store_breaker_open`
- `bijux_store_download_p95_seconds`
- `bijux_store_fetch_latency_p95_seconds`
- `bijux_store_open_p95_seconds`

## Alerts
- `AtlasOverloadSustained`
- `BijuxAtlasCacheThrash`
- `BijuxAtlasHigh5xxRate`
- `BijuxAtlasP95LatencyRegression`
- `BijuxAtlasStoreDownloadFailures`

## Dashboard Panels
- `Bulkhead Inflight by Class`
- `Bulkhead Saturation by Class`
- `Cache Hit Ratio`
- `Dataset Cache Hit/Miss`
- `Dataset Cache Size`
- `Endpoint Request/Response Size p95`
- `HTTP Request Rate by Route/Status`
- `HTTP p95 Latency by Route`
- `Queue Depth and Overload`
- `Rollout/Rollback View`
- `SLO Burn Rate (5xx, 5m/1h)`
- `SQLite Query p95 by Class`
- `Shed Rate by Reason`
- `Store Backend Fetch p95`
- `Store Open/Download p95`
- `Traffic Spike Drill View`
- `p99 Breakdown via Exemplars`

## Log Fields
- _none_

## Verification
```bash
make ops-observability-validate
```
