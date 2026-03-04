# Query Benchmark Summary Report

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@f9e6b3d92`

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
