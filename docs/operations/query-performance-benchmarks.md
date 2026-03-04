# Query Performance Benchmarks

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

Define the canonical query benchmark metric vocabulary and scenario matrix used by Atlas query performance evidence.

## Metrics

### Latency metrics

- `query_latency_ms_p50`: median end-to-end latency in milliseconds.
- `query_latency_ms_p95`: tail latency for sustained expected concurrency.
- `query_latency_ms_p99`: overload tail latency budget.
- `query_planner_latency_ms`: planning stage latency in milliseconds.
- `query_cursor_latency_ms`: cursor encode/decode latency in milliseconds.
- `query_response_serialize_latency_ms`: response serialization latency in milliseconds.

### Throughput metrics

- `query_throughput_ops_per_sec`: successful query operations per second.
- `query_throughput_rows_per_sec`: returned rows per second for range queries.

### Concurrency metrics

- `query_concurrency_level`: concurrent workers used in a benchmark run.
- `query_concurrency_saturation_ratio`: achieved throughput versus single-thread baseline.
- `query_concurrency_error_rate`: errors divided by total requests under concurrent load.

## Dataset size categories

- `small`: up to 100k records, single-node local benchmark.
- `medium`: 100k to 1M records, multi-shard local benchmark.
- `large`: 1M to 10M records, production-like benchmark.
- `x_large`: greater than 10M records, stress benchmark.

## Scenario matrix

- `point_lookup`: exact `gene_id` lookup.
- `region_query`: bounded genomic interval query.
- `prefix_search`: normalized prefix match on gene name.
- `projection_query`: minimal versus full projection comparison.
- `filter_query`: selective and low-selectivity filter combinations.
- `planner_latency`: parse and planning stage timing.
- `cursor_pagination`: first-page and next-page cursor flow.
- `serialization`: response serialization and JSON encoding overhead.
- `cache_hit`: repeated query on warm cache.
- `cache_miss`: first query for uncached request.
- `cache_eviction`: mixed workload with bounded cache capacity.
- `cache_warmup`: cold-to-warm transition behavior.

## Evidence mapping

- Query benchmark outputs are persisted under `artifacts/perf/`.
- Query benchmark baseline references are stored in `ops/report/`.
- Query benchmark suites live in `crates/bijux-atlas-query/benches/`.
