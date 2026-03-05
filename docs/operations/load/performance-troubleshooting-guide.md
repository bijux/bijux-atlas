# Load Performance Troubleshooting Guide

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Provide a reproducible debugging sequence for Atlas performance regressions.

## Workflow

1. Capture `load run` and `load compare` JSON artifacts.
2. Confirm SLO checks in `artifacts/load/slo-validation.json`.
3. Inspect resource pressure from `artifacts/load/resource-usage-heatmap.json`.
4. Verify deterministic and reproducible harness status.
5. Compare against baseline and trend report before changing infra.

## Typical Root Causes

- CPU saturation under burst query paths.
- Artifact cache pressure causing load latency spikes.
- Ingest/query contention reducing total throughput.
- Configuration changes increasing request fanout.

## Mitigation Checklist

- Increase replica count for CPU-bound scenarios.
- Increase cache budget when artifact pressure exceeds budget.
- Re-run `load compare` after each mitigation change.
