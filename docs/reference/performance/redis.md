# Redis Evaluation

## Exact Use Cases

1. Cross-pod rate limiting backend for IP/API-key limits.
2. Response cache for `gene_id` exact lookups only.

## Why These Two

- Both have bounded key cardinality and predictable TTL behavior.
- Both tolerate stale/missing cache entries without correctness risk.
- Both provide clear multi-pod value compared with in-process memory only.

## Explicit Non-Use Cases

- Do not cache region queries in Redis by default.
- Do not cache prefix scans in Redis by default.
- Do not use Redis for correctness-critical dataset integrity state.

## Safety and Failure Mode

- Redis is optional and config-gated.
- If Redis is down or slow, atlas falls back to local behavior (no hard dependency).
- Request handling continues with in-process limits/cache paths.

## Cache Keying Policy

- Gene exact cache key: `dataset_hash + gene_id + fields`.
- `dataset_hash` is derived from canonical dataset id string.

## Policy

Redis must never cache region queries unless there is an explicit bounded key-space design and documented explosion controls.

## Comparison Harness

Run:

```bash
./ops/load/scripts/compare_redis.sh
```

Output:
- `artifacts/perf/redis-compare/comparison.md`

Rate-limit fairness comparison:
- Run one pass with `ATLAS_ENABLE_REDIS_RATE_LIMIT=false` (per-pod buckets).
- Run one pass with `ATLAS_ENABLE_REDIS_RATE_LIMIT=true` (shared Redis buckets).
- Compare 429 distribution across pods/keys to evaluate fairness improvement.

## Shared rate-limit backend (optional)

Atlas supports optional Redis-backed shared rate limits across pods:
- `ATLAS_ENABLE_REDIS_RATE_LIMIT=true`
- `ATLAS_REDIS_URL=redis://...`

If Redis is unavailable, Atlas falls back to in-process limiting and continues serving.
This backend is optional and not required for correctness.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

Normative behavior and limits are listed here.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
