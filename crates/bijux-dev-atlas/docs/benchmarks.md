# Benchmarks

Benchmarking validates control-plane performance drift for critical surfaces.

The reusable benchmark model surface now lives in `bijux_dev_atlas::performance`.

Primary benchmark entrypoints:

- `cargo bench -p bijux-dev-atlas`
- `cargo bench -p bijux-dev-atlas --bench reproducibility`
- `cargo bench -p bijux-dev-atlas --bench security_supply_chain`

Benchmark governance, thresholds, and interpretation guidance are documented in
`docs/quality-system.md`.

Supporting crate docs:

- `docs/benchmarks/index.md`
- `docs/benchmarks/usage.md`
- `docs/benchmarks/architecture.md`
- `docs/benchmarks/troubleshooting.md`
- `docs/benchmarks/coverage-targets.md`
