# Load Failure Triage

- Owner: `bijux-atlas-operations`

## What

Runbook for diagnosing load suite failures and regressions.

## Symptoms

- `ops-load-full` exits non-zero.
- `score_k6.py` reports SLO violations.
- `validate_results.py` reports missing metrics/metadata.

## Metrics

- `bijux_requests_total`
- `bijux_request_duration_seconds`
- `bijux_dataset_cache_hits_total`

## Commands

```bash
$ make ops-load-full
$ python3 scripts/perf/score_k6.py
$ python3 scripts/perf/validate_results.py artifacts/perf/results
$ cat artifacts/e2e/k6/score.md
```

Expected output: failing scenario names and violating thresholds.

## Mitigations

- Re-run with stable host resources.
- Confirm dataset hash/release inputs.
- Compare against approved baseline in `ops/load/baselines/`.

## Rollback

- Revert recent perf-sensitive changes.
- Restore previous approved baseline if update was accidental.

## Postmortem checklist

- Record failing scenario and threshold.
- Record commit SHA/image digest from `.meta.json` sidecars.
- Attach report from `artifacts/load/reports/summary.md`.

## See also

- [Load Reproducibility](../load/reproducibility.md)
- [Performance Regression Policy](../observability/perf-regression-policy.md)
- `ops-load-full`
