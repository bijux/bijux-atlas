# SLO-Critical Metrics

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Metric meanings

- `http_requests_total{route,method,status,class}`: request outcomes by endpoint class.
- `http_request_duration_seconds_bucket{route,class}`: request latency histogram buckets.
- `atlas_overload_active`: overload state flag.
- `atlas_shed_total{reason,class}`: shed/rejected requests by reason and class.
- `atlas_cache_hits_total{cache="dataset"}` / `atlas_cache_misses_total{cache="dataset"}`: dataset cache behavior.
- `atlas_store_request_duration_seconds_bucket{backend}`: store latency histogram by backend.
- `atlas_store_errors_total{backend,class}`: store errors by backend and traffic class.
- `atlas_registry_refresh_age_seconds`: seconds since last successful registry refresh.
- `atlas_registry_refresh_failures_total`: registry refresh failure counter.
- `atlas_dataset_missing_total{dataset_hash}`: missing-dataset misses bucketed by hashed dataset id.
- `atlas_invariant_violations_total{invariant}`: semantic invariant violation counter.

## Troubleshooting tips

- Rising `atlas_overload_active` + `atlas_shed_total`: inspect queue depth and heavy concurrency limits.
- High `atlas_store_request_duration_seconds_bucket`: verify store backend health and network latency.
- Rising `atlas_registry_refresh_age_seconds`: check registry source health and freeze mode.
- Non-zero `atlas_invariant_violations_total`: run count/list parity checks before release.
- Rising `atlas_dataset_missing_total`: inspect publish/pin drift and cache warmup coverage.
