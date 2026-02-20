# Observability Surface

Generated from observability contract SSOT files:
- `ops/obs/contract/metrics-contract.json`
- `ops/obs/contract/alerts-contract.json`
- `ops/obs/contract/dashboard-panels-contract.json`
- `ops/obs/contract/logs-fields-contract.json`

## Metrics
- `atlas_bulkhead_inflight`
- `atlas_bulkhead_saturation`
- `atlas_cache_hits_total`
- `atlas_cache_misses_total`
- `atlas_client_requests_total`
- `atlas_dataset_missing_total`
- `atlas_invariant_violations_total`
- `atlas_overload_active`
- `atlas_policy_relaxation_active`
- `atlas_policy_violations_total`
- `atlas_registry_refresh_age_seconds`
- `atlas_registry_refresh_failures_total`
- `atlas_shed_total`
- `atlas_store_errors_total`
- `atlas_store_request_duration_seconds_bucket`
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
- `bijux_store_breaker_half_open_total`
- `bijux_store_breaker_open`
- `bijux_store_breaker_open_current`
- `bijux_store_download_p95_seconds`
- `bijux_store_fetch_latency_p95_seconds`
- `bijux_store_open_p95_seconds`
- `http_request_duration_seconds_bucket`
- `http_requests_total`

## Alerts
- `AtlasOverloadSustained`
- `BijuxAtlasCacheThrash`
- `BijuxAtlasCheapSloBurnFast`
- `BijuxAtlasCheapSloBurnMedium`
- `BijuxAtlasCheapSloBurnSlow`
- `BijuxAtlasHigh5xxRate`
- `BijuxAtlasOverloadSurvivalViolated`
- `BijuxAtlasP95LatencyRegression`
- `BijuxAtlasRegistryRefreshStale`
- `BijuxAtlasStandardSloBurnFast`
- `BijuxAtlasStandardSloBurnMedium`
- `BijuxAtlasStandardSloBurnSlow`
- `BijuxAtlasStoreBackendErrorSpike`
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
- `SLO Error Budget Burn (cheap/standard)`
- `SLO Health Status (cheap/standard)`
- `SQLite Query p95 by Class`
- `Shed Rate by Reason`
- `Store Backend Fetch p95`
- `Store Open/Download p95`
- `Traffic Spike Drill View`
- `p99 Breakdown via Exemplars`

## Log Fields
- `event_name`
- `level`
- `msg`
- `request_id`

## Verification
```bash
make ops-observability-validate
```
