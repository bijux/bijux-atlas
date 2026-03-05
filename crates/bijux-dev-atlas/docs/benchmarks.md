# Benchmarks

Benchmarking validates control-plane performance drift for critical surfaces.

Primary benchmark entrypoints:

- `cargo bench -p bijux-dev-atlas`
- `cargo bench -p bijux-dev-atlas --bench reproducibility`
- `cargo bench -p bijux-dev-atlas --bench security_supply_chain`

Benchmark governance, thresholds, and interpretation guidance are documented in
`docs/quality-system.md`.
