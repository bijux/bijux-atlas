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
- Startup warmup jitter (`startup_warmup_jitter_max_ms`) reduces pod stampede risk.
- Datasets are quarantined after repeated integrity failures (`quarantine_after_corruption_failures`).
- Quarantine and retry budgets prevent infinite re-download loops for bad artifacts.
- Emergency global breaker can reject non-health traffic (`emergency_global_breaker`).
- Request queue depth is capped (`max_request_queue_depth`) to prevent unbounded backlog.
- Store download concurrency is globally capped (`max_concurrent_downloads`).
- Store retry budget and store circuit-breaker thresholds are configurable for herd protection.
- Retry budget is enforced globally and per dataset to avoid infinite retries on a single bad artifact.
- On request timeout, optional policy can continue background download for pinned warmup datasets.
- Optional shard caches are selected by seqid for region queries and evicted with dataset cleanup.
- Open shard concurrency is capped (`max_open_shards_per_pod`) to protect FD/memory limits.
- Redis is tier-2 optional only. If Redis is slow/down, request handling falls back to local paths.
- Redis response cache is limited to exact `gene_id` lookups with strict key-size/cardinality/TTL caps.
- Redis operations are guarded by timeout, retry budget, and local circuit breaker.
- Cache hit ratio and disk budget are validated in ops via `make ops-cache-status`.

Read-only sqlite pragma profile:
- `PRAGMA query_only=ON`
- `PRAGMA journal_mode=OFF`
- `PRAGMA synchronous=OFF`
- `PRAGMA temp_store=MEMORY`
- `PRAGMA cache_size=-<configured KiB>`
- `PRAGMA mmap_size=<configured bytes>`
- For shard DB opens, mmap uses a per-shard tuned budget derived from global setting.

Benchmarks:
- Cold start: `make ops-perf-cold-start`
- Warm start: `make ops-perf-warm-start`

## Registry Federation

- Multiple registries are configured with `ATLAS_REGISTRY_SOURCES` and optional `ATLAS_REGISTRY_PRIORITY`.
- Catalog refresh is TTL-governed by `ATLAS_REGISTRY_TTL_MS`.
- `ATLAS_REGISTRY_FREEZE_MODE=true` pauses refresh and serves last known merged catalog.
- Registry health is exposed at `/debug/registry-health` when debug endpoints are enabled.
