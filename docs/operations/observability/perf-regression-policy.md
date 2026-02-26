# Performance Regression Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Nightly performance suite is authoritative for latency regressions.

## Inputs
- k6 suite outputs under `artifacts/perf/results/*.summary.json`
- Thresholds in `configs/perf/k6-thresholds.v1.json`
- Generated summary in `artifacts/perf/report.md`

## Gate
- If any suite exceeds configured `p95_ms` or `fail_rate`, nightly fails.
- A regression artifact is written to `artifacts/perf/regression.txt`.
- Nightly workflow opens/updates an issue with regression details.

## Scope
- Cold start benchmark
- Warm steady-state
- Spike burst ramp
- Mixed query distribution (80% cheap / 20% heavy)
- Cache stampede scenario
- Store-outage-mid-spike scenario
- Soak test memory drift signal

## See also

- `ops-ci`
