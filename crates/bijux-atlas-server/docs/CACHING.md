# Caching

`DatasetCacheManager` invariants:

- Single-flight download per dataset id.
- Verify checksum before publish to cache path.
- Atomic publish: write temp then rename.
- Respect pinned datasets during eviction.
- Enforce dataset count and disk budget caps deterministically.
- Circuit breaker blocks repeated failing opens for configured duration.
- Cached-only mode never reaches network/store.
- Per-dataset query semaphore caps concurrent sqlite work per dataset.
- Short-TTL coalesced query cache is enabled for heavy identical requests.
- Optional heavy-query shed mode returns 503 for heavy class when p95 exceeds threshold.
- Startup cache warmup is deterministic (sorted dataset ids) and bounded by `startup_warmup_limit`.
- Store download concurrency is globally capped (`max_concurrent_downloads`).
- Store retry budget and store circuit-breaker thresholds are configurable for herd protection.
- Retry budget is enforced globally and per dataset to avoid infinite retries on a single bad artifact.
- On request timeout, optional policy can continue background download for pinned warmup datasets.
- Optional shard caches are selected by seqid for region queries and evicted with dataset cleanup.
- Open shard concurrency is capped (`max_open_shards_per_pod`) to protect FD/memory limits.

Read-only sqlite pragma profile:
- `PRAGMA query_only=ON`
- `PRAGMA journal_mode=OFF`
- `PRAGMA synchronous=OFF`
- `PRAGMA temp_store=MEMORY`
- `PRAGMA cache_size=-<configured KiB>`
- `PRAGMA mmap_size=<configured bytes>`
- For shard DB opens, mmap uses a per-shard tuned budget derived from global setting.
