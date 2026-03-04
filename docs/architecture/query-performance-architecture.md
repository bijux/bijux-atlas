# Query Performance Architecture

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`

## Purpose

Describe how Atlas query performance evidence is produced, validated, and enforced.

## Architecture Layers

- Benchmark runtime: Criterion benchmark suites in `crates/bijux-atlas-query/benches/`
- Evidence fixtures: baseline and golden JSON in `crates/bijux-atlas-query/tests/goldens/perf/`
- Regression validation: deterministic tests in `crates/bijux-atlas-query/tests/query_benchmark_regression.rs`
- CI enforcement: workflow lane in `.github/workflows/query-benchmark-ci.yml`
- Operations view: dashboard and runbook docs under `docs/operations/`

## Evidence Flow

1. Bench scenario runs for lookup, filter, cache, routing, and index workloads.
2. Results are reviewed against baseline and golden fixture expectations.
3. Regression tests enforce required scenario and check coverage.
4. CI executes regression tests on pull request and mainline updates.

## Non-goals

- This page does not define ingest benchmark architecture.
- This page does not replace runtime observability architecture.
