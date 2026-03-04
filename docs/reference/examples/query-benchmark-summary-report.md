# Query Benchmark Summary Report Example

- Owner: `reference`
- Stability: `stable`

## Summary

- Profile: `query-benchmark-baseline`
- Dataset tier: `medium`
- Revision: `main@f9e6b3d92`
- Status: `pass`

## Scenario Results

- `point_lookup`: p95 `14ms`, throughput `6200 ops/s`
- `region_query`: p95 `61ms`, throughput `29000 rows/s`
- `query_concurrency`: p95 `87ms`, throughput `11200 ops/s`
- `query_cache_hit`: p95 `2ms`
- `query_cache_miss`: p95 `19ms`
- `query_shard_routing`: p95 `6ms`
- `query_index_scan`: p95 `12ms`

## Notes

- No regression threshold breaches were detected.
- Cache miss latency improved after index and planner updates.
