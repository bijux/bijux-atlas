# Query Benchmark Dashboards

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@f9e6b3d92`

## Purpose

Define the dashboard panels used to visualize query benchmark evidence and compare runs over time.

## Required Dashboard Panels

- `query latency`: p50, p95, p99 per scenario.
- `query throughput`: operations per second and rows per second.
- `query concurrency`: active workers, saturation ratio, and error rate.
- `query planner`: planner and normalization latency.
- `query cache`: hit rate, miss rate, eviction count, and warmup progress.
- `query routing`: shard selection distribution and fanout costs.
- `query index`: indexed path latency and index-scan latency.

## Required Dashboard Filters

- Dataset tier (`small`, `medium`, `large`, `x_large`)
- Scenario (`point_lookup`, `region_query`, `prefix_search`, `projection_query`, `filter_query`)
- Build revision
- Benchmark profile (`baseline`, `candidate`)

## Evidence Output

- Store screenshots or exports under `artifacts/perf/query-dashboards/`.
- Store the final comparison summary under `artifacts/perf/query-summary.json`.
