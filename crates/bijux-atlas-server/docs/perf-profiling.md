# Perf Profiling

## pprof / Flamegraph Workflow (Non-default)

1. Start server with representative dataset cache.
2. Replay load with pinned suite:
   `BASE_URL=http://127.0.0.1:8080 k6 run ops/load/k6/suites/mixed-80-20.js`
3. Collect profile externally (Linux perf + flamegraph tooling).
4. Save outputs in `artifacts/benchmarks/flamegraph/`.

## Allocator Regression Signal

Use allocator-level telemetry from runtime/container metrics during soak tests to catch unexpected allocation growth on hot paths.
