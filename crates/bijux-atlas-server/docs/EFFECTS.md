# Effects

This crate owns runtime effects for Atlas serving:

- Network I/O: HTTP server endpoints and optional store backends.
- Disk I/O: dataset cache root, manifest/sqlite writes, atomic rename.
- Time effects: request timeouts, circuit-breaker windows, eviction TTL.
- Concurrency effects: per-query-class bulkheads and connection semaphores.

Other crates should remain pure or narrowly scoped to their boundaries.
