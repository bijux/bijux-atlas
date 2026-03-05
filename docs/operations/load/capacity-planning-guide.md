# Capacity Planning Guide

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define how to turn load evidence into actionable capacity decisions for Atlas environments.

## Inputs

- Baseline snapshot: `artifacts/load/baseline-snapshot.json`
- Current measurement: `artifacts/load/current-measurement.json`
- Capacity estimation report: `artifacts/load/capacity-estimation-report.json`
- Capacity summary: `artifacts/load/capacity-summary.json`
- Capacity recommendation: `artifacts/load/capacity-recommendation.json`

## Decision Flow

1. Run `bijux-dev-atlas load baseline --format json` for the reference profile.
2. Run `bijux-dev-atlas load run --format json` under candidate conditions.
3. Run `bijux-dev-atlas load compare --format json`.
4. Reject promotion on regression status or failing SLO checks.
5. Apply scaling and cache adjustments from `capacity-recommendation` before retry.

## Promotion Gates

- `latency_p99_ms` stays within defined threshold budget.
- `error_rate_pct` remains below 1.0.
- `cpu_utilization_pct` remains below environment ceiling.
- `artifact_cache_pressure_pct` remains below 80.
