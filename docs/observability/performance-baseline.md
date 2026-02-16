# Atlas Performance Baseline

## Baseline Targets

- `v1/genes/count` p95 <= 300 ms under steady mixed load.
- `v1/genes` p95 <= 800 ms for normal query mix.
- cold-start first successful query <= 2.5 s for pinned warm-cache profile.

## Benchmark Artifacts

- 1000 QPS load scenario: `load/k6/atlas_1000qps.js`
- cold-start benchmark script: `scripts/perf/cold_start_benchmark.sh`
- memory profile guide: `docs/runbooks/MEMORY_PROFILE_UNDER_LOAD.md`

All benchmark outputs should be saved under `artifacts/benchmarks/`.
