# Ops Budgets

Canonical SSOT: `configs/ops/budgets.json`.

## smoke
- `max_duration_seconds`: hard limit for `ops-smoke` runtime.

## root_local
- `lane_max_duration_seconds`: per-lane max duration budgets for root/root-local lanes.
- `warning_band_ratio`: warning threshold ratio for near-failing budgets.

## k6_latency
- `default_p95_ms`: default p95 latency threshold.
- `default_p99_ms`: default p99 latency threshold.

## cold_start
- `max_p99_ms`: cold-start p99 upper bound.

## cache
- `max_download_bytes`: maximum allowed cache fetch budget.
- `max_dataset_count`: maximum dataset count budget.

## metric_cardinality
- `max_series_per_metric`: maximum series budget per metric.

Budget changes must include rationale in the PR description and, when temporary, an entry in `configs/policy/budget-relaxations.json`.
