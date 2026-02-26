# Ops Budgets

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Canonical SSOT: `configs/ops/budgets.json`.

## What

Defines the single source of truth for runtime and performance budgets enforced by ops checks.

## Why

Prevents hidden threshold drift and keeps failure criteria consistent across local and CI lanes.

## Contracts

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

## Failure modes

Embedded constants in scripts, missing budget keys, or undocumented budget changes can cause unreviewed regressions.

## How to verify

```bash
make lane-configs-policies
```

Expected output: config/policy lane passes, including budget schema and budget enforcement checks.

## See also

- `configs/ops/budgets.json`
- `configs/policy/budget-relaxations.json`

Budget changes must include rationale in the PR description and, when time-bound, an entry in `configs/policy/budget-relaxations.json`.
