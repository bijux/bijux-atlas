# Atlas Performance Baseline

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Baseline Targets

- `v1/genes/count` p95 <= 300 ms under steady mixed load.
- `v1/genes` p95 <= 800 ms for normal query mix.
- cold-start first successful query <= 2.5 s for pinned warm-cache profile.

## Benchmark Artifacts

- 1000 QPS load scenario: `ops/load/k6/atlas_1000qps.js`
- cold-start benchmark script: `crates/bijux-dev-atlas/src/commands/ops/load/run/cold_start_benchmark.py` (invoke via `bijux dev atlas`)
- cache manager bench: `cargo bench -p bijux-atlas-server --bench cache_manager`
- mmap experiment (non-CI): `cargo test -p bijux-atlas-server mmap_read_only_experiment_baseline -- --ignored`
- memory profile guide: `docs/runbooks/memory-profile-under-load.md`

All benchmark outputs should be saved under `artifacts/benchmarks/`.

## See also

- `ops-ci`
