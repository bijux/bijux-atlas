# Benchmark Usage

Use `bijux-dev-atlas perf` commands as the benchmark runner surface and `bijux_dev_atlas::performance` as the reusable model surface.

## Core flow

1. Validate benchmark contracts:
`bijux-dev-atlas perf validate --format json`
2. Run benchmark scenario:
`bijux-dev-atlas perf run --scenario gene-lookup --format json`
3. Compare with baseline:
`bijux-dev-atlas perf diff ops/report/gene-lookup-baseline.json artifacts/perf/gene-lookup-load.json --format json`

## Rust API surface

- `bijux_dev_atlas::performance::BenchmarkConfig`
- `bijux_dev_atlas::performance::DatasetRegistry`
- `bijux_dev_atlas::performance::BenchmarkResult`
- `bijux_dev_atlas::performance::compare_results`
- `bijux_dev_atlas::performance::reproducibility_ok`

## Produced artifacts

- `artifacts/perf/gene-lookup-load.json`
- `artifacts/benchmarks/gene-lookup-result.json`
- `artifacts/benchmarks/gene-lookup-result.csv`
- `artifacts/benchmarks/gene-lookup-summary.json`
- `artifacts/benchmarks/gene-lookup-history.json`
