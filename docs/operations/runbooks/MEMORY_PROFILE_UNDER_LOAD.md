# Memory Profiling Under Load

## Goal

Capture heap/memory growth under realistic query load.

## Procedure

1. Start server with representative dataset cache settings.
2. Run load:
   - `k6 run load/k6/atlas_1000qps.js`
3. Profile memory:
   - Linux: `heaptrack ./target/release/atlas-server`
   - macOS: `xcrun xctrace record --template 'Allocations' --launch ./target/release/atlas-server`
4. Export artifacts to `artifacts/benchmarks/memory/` and compare peak RSS + allocation hotspots.

## Acceptance

- No unbounded RSS growth during 10+ minute steady load.
- Top allocations are explainable and stable between releases.
