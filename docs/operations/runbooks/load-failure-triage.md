# Load Failure Triage

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
$ python3 scripts/perf/score_k6.py
$ python3 scripts/perf/validate_results.py artifacts/perf/results
$ cat artifacts/ops/e2e/k6/score.md
```

Expected output: failing scenario names and violating thresholds.

## Expected outputs

- `scripts/perf/score_k6.py` reports either `k6 SLO score passed` or explicit suite violations.
- `scripts/perf/validate_results.py` reports `load result contract validation passed`.
- `artifacts/ops/load/reports/summary.md` includes suite latency/error rows with metadata fields.

## Mitigations

- Re-run with stable host resources.
- Confirm dataset hash/release inputs.
- Compare against approved baseline in `ops/load/baselines/`.
- Validate suite contracts with `python3 scripts/perf/validate_suite_manifest.py`.

Canonical suite names:
- `mixed`
- `cheap-only-survival`
- `warm-steady-state-p99`
- `cold-start-p99`
- `cold-start-prefetch-5pods`
- `stampede`
- `store-outage-mid-spike`
- `noisy-neighbor-cpu-throttle`
- `pod-churn`
- `response-size-abuse`
- `multi-release`
- `sharded-fanout`
- `diff-heavy`
- `mixed-gene-sequence`
- `load-under-rollout`
- `load-under-rollback`
- `soak-30m`
- `redis-optional` (experiment, opt-in only)

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
