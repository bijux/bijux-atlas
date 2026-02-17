# Load CI Policy

- Owner: `bijux-atlas-operations`

## What

Defines which load suites run on PR versus nightly pipelines.

## Why

Balances fast feedback with deep regression detection.

## Scope

k6-based load suites and scoring jobs.

## Non-goals

Does not define endpoint correctness tests.

## Contracts

- PR tier: smoke subset only, deterministic short runtime.
- Nightly tier: full suites including spike, churn, outage, and soak-linked scenarios.
- SLO scoring must use `scripts/perf/score_k6.py` with `configs/slo/slo.json`.
- PR smoke scenarios include `mixed.json` and `response_size_guardrails.json`.
- Nightly scenarios include `spike.json`, `stampede.json`, `store_outage.json`, and `pod_churn.json`.

## Failure modes

Too-light PR coverage misses severe regressions; too-heavy PR coverage blocks iteration speed.

## How to verify

```bash
$ ./scripts/perf/run_suite.sh smoke
$ ./scripts/perf/run_nightly_perf.sh
```

Expected output: smoke and nightly suites complete with scored reports.

## See also

- [Load Suites](suites.md)
- [k6 Harness](k6.md)
- [Perf Regression Policy](../observability/perf-regression-policy.md)
- `ops-ci`
