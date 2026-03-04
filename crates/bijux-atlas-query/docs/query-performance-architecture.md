# Query Performance Architecture

- Owner: `bijux-atlas-query`
- Stability: `stable`

## Purpose

Describe the architecture used to measure query performance and keep evidence reproducible.

## Components

- Benchmark suites in `crates/bijux-atlas-query/benches/`
- Baseline and golden fixtures in `crates/bijux-atlas-query/tests/goldens/perf/`
- Regression tests in `crates/bijux-atlas-query/tests/query_benchmark_regression.rs`
- CI lane in `.github/workflows/query-benchmark-ci.yml`

## Data Flow

1. Bench suite executes deterministic query scenario.
2. Bench output is compared with baseline expectations.
3. Regression tests validate required scenario and check coverage.
4. CI enforces fixture coverage on pull requests and `main`.

## Invariants

- Scenario naming is stable and explicit.
- Baseline and golden files are versioned evidence.
- Bench changes and fixture changes are reviewed together.
