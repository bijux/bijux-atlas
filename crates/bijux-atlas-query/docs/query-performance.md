# Query Performance

- Owner: `bijux-atlas-query`
- Stability: `stable`

## Purpose

Provide the canonical query performance reference for scenarios, metrics, and benchmark suites.

## Benchmark Suites

- `benches/query_patterns.rs`
- `benches/query_concurrency.rs`
- `benches/query_stages.rs`
- `benches/query_planner_and_serialization.rs`
- `benches/query_cache.rs`
- `benches/query_routing_and_index.rs`

## Metrics

- Latency: `query_latency_ms_p50`, `query_latency_ms_p95`, `query_latency_ms_p99`
- Throughput: `query_throughput_ops_per_sec`, `query_throughput_rows_per_sec`
- Concurrency: `query_concurrency_level`, `query_concurrency_saturation_ratio`, `query_concurrency_error_rate`

## Coverage Surface

- Lookup and filter paths (`point_lookup`, `prefix_search`, `region_query`, `filter_query`)
- Planner and normalization costs
- Cursor pagination and response serialization
- Cache hot/cold behavior
- Shard routing and fanout simulation
- Index scan and covering index behavior
