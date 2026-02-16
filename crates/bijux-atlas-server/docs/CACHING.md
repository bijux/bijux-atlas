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

Read-only sqlite pragma profile:
- `PRAGMA query_only=ON`
- `PRAGMA journal_mode=OFF`
- `PRAGMA synchronous=OFF`
- `PRAGMA temp_store=MEMORY`
- `PRAGMA cache_size=-<configured KiB>`
- `PRAGMA mmap_size=<configured bytes>`
