# Metrics Contract

- Owner: `docs-governance`

## What

Defines the `Metrics Contract` registry contract.

## Why

Prevents drift between SSOT JSON, generated code, and operational consumers.

## Scope

Applies to producers and consumers of this registry.

## Non-goals

Does not define implementation internals outside this contract surface.

## Contracts

- `atlas_bulkhead_inflight` labels: class, dataset, subsystem, version
- `atlas_bulkhead_saturation` labels: class, dataset, subsystem, version
- `atlas_cache_hits_total` labels: cache, dataset, subsystem, version
- `atlas_cache_misses_total` labels: cache, dataset, subsystem, version
- `atlas_client_requests_total` labels: client_type, dataset, subsystem, user_agent_family, version
- `atlas_dataset_missing_total` labels: dataset, dataset_hash, subsystem, version
- `atlas_invariant_violations_total` labels: dataset, invariant, subsystem, version
- `atlas_overload_active` labels: dataset, subsystem, version
- `atlas_policy_relaxation_active` labels: dataset, mode, subsystem, version
- `atlas_policy_violations_total` labels: dataset, policy, subsystem, version
- `atlas_registry_refresh_age_seconds` labels: dataset, subsystem, version
- `atlas_registry_refresh_failures_total` labels: dataset, subsystem, version
- `atlas_shed_total` labels: class, dataset, reason, subsystem, version
- `atlas_store_errors_total` labels: backend, class, dataset, subsystem, version
- `atlas_store_request_duration_seconds_bucket` labels: backend, dataset, le, subsystem, version
- `bijux_dataset_count` labels: dataset, subsystem, version
- `bijux_dataset_disk_usage_bytes` labels: dataset, subsystem, version
- `bijux_dataset_hits` labels: dataset, subsystem, version
- `bijux_dataset_misses` labels: dataset, subsystem, version
- `bijux_disk_io_latency_p95_ns` labels: dataset, subsystem, version
- `bijux_errors_total` labels: code, dataset, subsystem, version
- `bijux_fs_space_pressure_events_total` labels: dataset, subsystem, version
- `bijux_http_request_latency_p95_seconds` labels: dataset, route, subsystem, version
- `bijux_http_request_size_p95_bytes` labels: dataset, route, subsystem, version
- `bijux_http_requests_total` labels: class, dataset, method, route, status, subsystem, version
- `bijux_http_response_size_p95_bytes` labels: dataset, route, subsystem, version
- `bijux_overload_shedding_active` labels: dataset, subsystem, version
- `bijux_request_queue_depth` labels: dataset, subsystem, version
- `bijux_request_stage_latency_p95_seconds` labels: dataset, stage, subsystem, version
- `bijux_runtime_policy_hash` labels: dataset, subsystem, version
- `bijux_sqlite_query_latency_p95_seconds` labels: dataset, query_type, subsystem, version
- `bijux_store_breaker_half_open_total` labels: dataset, subsystem, version
- `bijux_store_breaker_open` labels: dataset, subsystem, version
- `bijux_store_breaker_open_current` labels: dataset, subsystem, version
- `bijux_store_download_p95_seconds` labels: dataset, subsystem, version
- `bijux_store_fetch_latency_p95_seconds` labels: backend, dataset, subsystem, version
- `bijux_store_open_p95_seconds` labels: dataset, subsystem, version
- `http_request_duration_seconds_bucket` labels: class, dataset, le, route, subsystem, version
- `http_requests_total` labels: class, dataset, method, route, status, subsystem, version

Label cardinality rules:
- User-controlled values must not be used as metric labels.
- Allowed dynamic labels are constrained by `ops/observe/contract/metrics-contract.json`.

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

```json
{
  "labels": [
    "class",
    "dataset",
    "subsystem",
    "version"
  ],
  "metric": "atlas_bulkhead_inflight"
}
```

## How to verify

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](contracts-index.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
