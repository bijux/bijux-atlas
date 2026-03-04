# Benchmark Usage

Use `bijux-dev-atlas perf` commands as the benchmark runner surface.

## Core flow

1. Validate benchmark contracts:
`bijux-dev-atlas perf validate --format json`
2. Run benchmark scenario:
`bijux-dev-atlas perf run --scenario gene-lookup --format json`
3. Compare with baseline:
`bijux-dev-atlas perf diff ops/report/gene-lookup-baseline.json artifacts/perf/gene-lookup-load.json --format json`

## Produced artifacts

- `artifacts/perf/gene-lookup-load.json`
- `artifacts/benchmarks/gene-lookup-result.json`
- `artifacts/benchmarks/gene-lookup-result.csv`
- `artifacts/benchmarks/gene-lookup-summary.json`
- `artifacts/benchmarks/gene-lookup-history.json`
