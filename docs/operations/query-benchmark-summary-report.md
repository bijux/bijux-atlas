# Query Benchmark Summary Report

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

Define the report structure used to summarize query benchmark runs for review and release evidence.

## Required Fields

- `run_id`
- `revision`
- `dataset_tier`
- `profile`
- `status`
- `scenarios[]` with per-scenario latency and throughput values
- `regressions[]` listing threshold breaches
- `notes`

## Artifact Location

- `artifacts/perf/query-summary.json`
- Optional markdown rendering: `artifacts/perf/query-summary.md`

## Example

See `docs/reference/examples/query-benchmark-summary-report.md`.
