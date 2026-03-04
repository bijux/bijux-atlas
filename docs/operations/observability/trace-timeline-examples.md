# Trace Timeline Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@f569762c0`
- Reason to exist: provide concrete trace timeline examples for common failure triage.

## Healthy query timeline

1. `http.request`
2. `query_plan`
3. `query_execution`
4. `cursor_generation`

Expected shape: planning and execution dominate, no repeated shard routing retries.

## Slow shard timeline

1. `http.request`
2. `query_plan`
3. `shard_routing`
4. `query_execution`

Expected shape: shard routing expands and total span duration exceeds slow-query threshold.

## Cache hit timeline

1. `http.request`
2. `cache_lookup_hot_query`
3. response serialization

Expected shape: no SQL execution spans.
