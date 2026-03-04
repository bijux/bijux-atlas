# Query Benchmark Examples

- Owner: `reference`
- Stability: `stable`

## Example Commands

```bash
cargo bench -p bijux-atlas-query query_patterns
cargo bench -p bijux-atlas-query query_planner_and_serialization
cargo bench -p bijux-atlas-query query_cache
cargo bench -p bijux-atlas-query query_routing_and_index
```

## Example Scenario Mapping

- `query_patterns`: point lookup, region query, prefix search, projection, filter
- `query_planner_and_serialization`: planner latency, normalization, cursor generation, response serialization, JSON encoding
- `query_cache`: cache performance, hit, miss, eviction, warmup
- `query_routing_and_index`: shard routing, fanout simulation, planner complexity, index scan, covering index, cursor pagination
