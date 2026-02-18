# Perf Baseline

SQLite tuning experiments are captured with these knobs:
- `page_size=4096` (ingest build-time fixed value)
- `mmap_size` configurable at serve-time via `ATLAS_SQLITE_MMAP_BYTES`
- `cache_size` configurable at serve-time via `ATLAS_SQLITE_CACHE_KIB`

Baseline capture:
- Run `cargo test -p bijux-atlas-server --test p99-regression -- --nocapture`.
- Track `latency_regression_guard_p95_under_threshold`.
- Track `db_open_is_cheap_regression_guard` separately from query benches.

Allocator policy:
- Default allocator: system.
- Optional allocator: `jemalloc` feature (`cargo run -p bijux-atlas-server --features jemalloc`).
- Compare both with identical load profile before production changes.
