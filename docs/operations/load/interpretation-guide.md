# Load Test Interpretation Guide

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Purpose

Provide a deterministic interpretation model for baseline, run, and comparison artifacts.

## Key Outputs

- `artifacts/load/comparison-report.json`
- `artifacts/load/trend-analysis.json`
- `artifacts/load/performance-trend-report.json`
- `artifacts/load/performance-stability-index.json`

## Interpretation Rules

1. `regression_detected=true` is release blocking unless an approved exception exists.
2. `trend=degrading` requires root-cause analysis before deployment.
3. Stability index below `0.80` requires additional stress iterations.
4. Throughput drop with stable latency indicates capacity saturation risk.
5. Latency increase with stable throughput indicates contention or dependency drift.

## Escalation

- Open incident triage when two consecutive comparisons show regression.
- Record mitigation evidence in load evidence bundle artifacts.
