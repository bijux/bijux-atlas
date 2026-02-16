# Caching

`DatasetCacheManager` invariants:

- Single-flight download per dataset id.
- Verify checksum before publish to cache path.
- Atomic publish: write temp then rename.
- Respect pinned datasets during eviction.
- Enforce dataset count and disk budget caps deterministically.
- Circuit breaker blocks repeated failing opens for configured duration.
- Cached-only mode never reaches network/store.
