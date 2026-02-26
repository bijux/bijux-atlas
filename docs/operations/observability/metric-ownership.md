# Metric Ownership

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-server` + `bijux-atlas-store`

## What

Defines which runtime component emits each observability metric family.

## Why

Avoids ownership ambiguity and makes oncall triage map directly from metric to crate/code path.

## Contracts

- `bijux_http_*`, `atlas_bulkhead_*`, `atlas_shed_total`:
  - Emitter: `crates/bijux-atlas-server/src/telemetry/metrics_endpoint.rs`
  - Input signals: request metrics, bulkhead semaphores, shedding counters
- `bijux_store_*`:
  - Emitter: `crates/bijux-atlas-server/src/telemetry/metrics_endpoint.rs`
  - Source data: `DatasetCacheManager` storage/lifecycle metrics
- `atlas_policy_*`:
  - Emitter: `crates/bijux-atlas-server/src/telemetry/metrics_endpoint.rs`
  - Source data: policy middleware rejection counters and startup policy mode/hash
- Span contract (`TRACE_SPANS.json`) ownership:
  - Request spans in handlers (`crates/bijux-atlas-server/src/http/*`)
  - Cache/store spans in cache manager (`crates/bijux-atlas-server/src/runtime/dataset_cache_manager_storage.rs`)

## Failure modes

- Missing ownership causes duplicate emitters, metric drift, and ambiguous incident triage.
- Moving emitters without updating this map breaks contract validation expectations.

## How to verify

```bash
make ssot-check
make ops-metrics-check
make ops-traces-check
```

Expected output: contract checks pass and required telemetry names are present.
