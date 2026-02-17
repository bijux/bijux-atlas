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

- `bijux_dataset_count` labels: dataset, subsystem, version
- `bijux_dataset_disk_usage_bytes` labels: dataset, subsystem, version
- `bijux_dataset_hits` labels: dataset, subsystem, version
- `bijux_dataset_misses` labels: dataset, subsystem, version
- `bijux_errors_total` labels: code, dataset, subsystem, version
- `bijux_http_request_latency_p95_seconds` labels: dataset, route, subsystem, version
- `bijux_http_requests_total` labels: dataset, route, status, subsystem, version
- `bijux_overload_shedding_active` labels: dataset, subsystem, version
- `bijux_request_stage_latency_p95_seconds` labels: dataset, stage, subsystem, version
- `bijux_sqlite_query_latency_p95_seconds` labels: dataset, query_type, subsystem, version
- `bijux_store_breaker_open` labels: dataset, subsystem, version
- `bijux_store_download_p95_seconds` labels: dataset, subsystem, version
- `bijux_store_open_p95_seconds` labels: dataset, subsystem, version

Label cardinality rules:
- User-controlled values must not be used as metric labels.
- Allowed dynamic labels are constrained by `ops/observability/contract/metrics-contract.json`.

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

```json
{
  "labels": [
    "dataset",
    "subsystem",
    "version"
  ],
  "metric": "bijux_dataset_count"
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
