# Load Failure Triage

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Runbook for diagnosing load suite failures and regressions.

## Symptoms

- `ops-load-full` exits non-zero.
- `ops-load-ci` or `ops-load-nightly` exits non-zero.
- `score_k6.py` reports SLO violations.
- `validate_results.py` reports missing metrics/metadata.

## Metrics

- `bijux_http_requests_total`
- `bijux_http_request_latency_p95_seconds`
- `bijux_dataset_hits`

## Commands

```bash
$ make ops-load-ci
$ make ops-load-nightly
$ make ops-perf-report
$ make ops-load-manifest-validate
$ cat artifacts/ops/e2e/k6/score.md
```

Expected output: failing scenario names and violating thresholds.

## Expected outputs

- `make ops-perf-report` reports either `k6 SLO score passed` or explicit suite violations.
- `make ops-load-manifest-validate` reports `load suite manifest validation passed`.
- `artifacts/ops/load/reports/summary.md` includes suite latency/error rows with metadata fields.

## Mitigations

- Re-run with stable host resources.
- Confirm dataset hash/release inputs.
- Compare against approved baseline in `ops/load/baselines/`.
- Validate suite contracts with `make ops-load-manifest-validate`.

## Suite Categories

- Availability under pressure: `cheap-only-survival`, `pod-churn`, `store-outage-under-spike`
- Latency-heavy workload: `diff-heavy`, `mixed-gene-sequence`, `multi-release`
- Abuse and guardrails: `response-size-abuse`, `stampede`
- Spike overload proof: `spike-overload-proof`
- Long-run stability: `soak-30m`

Canonical suite names:
- `mixed`
- `cheap-only-survival`
- `warm-steady-state-p99`
- `cold-start-p99`
- `cold-start-prefetch-5pods`
- `stampede`
- `store-outage-under-spike`
- `noisy-neighbor-cpu-throttle`
- `pod-churn`
- `spike-overload-proof`
- `response-size-abuse`
- `multi-release`
- `sharded-fanout`
- `diff-heavy`
- `mixed-gene-sequence`
- `load-under-rollout`
- `load-under-rollback`
- `soak-30m`
- `redis-optional` (experiment, opt-in only)
- `hpa-validation-short`

## Alerts

- `BijuxAtlasP95LatencyRegression`

## Rollback

- Revert recent perf-sensitive changes.
- Restore previous approved baseline if update was accidental.

## Postmortem checklist

- Record failing scenario and threshold.
- Record commit SHA/image digest from `.meta.json` sidecars.
- Attach report from `artifacts/ops/load/reports/summary.md`.

## See also

- [Load Reproducibility](../load/reproducibility.md)
- [Performance Regression Policy](../observability/perf-regression-policy.md)
- `ops-load-full`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
