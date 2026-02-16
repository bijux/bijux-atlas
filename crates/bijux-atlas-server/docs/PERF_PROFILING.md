# Perf Profiling

## Feature-Gated Build

Enable profiling feature when collecting flamegraphs:

```bash
cargo run -p bijux-atlas-server --features profiling
```

## pprof / Flamegraph Workflow (Non-default)

1. Start server with representative dataset cache.
2. Replay load with pinned suite:
   `BASE_URL=http://127.0.0.1:8080 k6 run load/k6/suites/mixed_80_20.js`
3. Collect profile externally (Linux perf + inferno or pprof tooling).
4. Save outputs in `artifacts/benchmarks/flamegraph/`.

## Allocator Regression Signal

When compiled with `--features jemalloc`, `/metrics` exports:

- `bijux_allocator_allocated_bytes{allocator="jemalloc"}`

Track this under soak tests to catch unexpected allocation growth on hot paths.
