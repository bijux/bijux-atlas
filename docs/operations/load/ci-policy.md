# Load CI Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

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

- PR tier: smoke subset only, deterministic short runtime via `make ops-load-smoke`.
- Nightly tier: full suites (including soak) via `make ops-load-nightly` and `make ops-load-full`.
- SLO scoring must use `ops-load-ci` with `configs/slo/slo.json`.
- Suite manifest SSOT: `ops/load/suites/suites.json`.
- Suite manifest must validate against `ops/load/contracts/suite-schema.json`.
- Redis experiment suite runs only when `ATLAS_ENABLE_REDIS_EXPERIMENT=1`.
- Baseline updates must use `make ops-perf-baseline-update` and satisfy baseline policy gate.
- PR smoke scenarios include `mixed.json`.
- Nightly scenarios include `spike.json`, `stampede.json`, `store-outage-under-spike.json`, `pod-churn.json`, `diff-heavy.json`, and `soak-30m.json`.

## Failure modes

Too-light PR coverage misses severe regressions; too-heavy PR coverage blocks iteration speed.

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
```

Expected output: smoke and nightly suites complete with scored reports.

## See also

- [Load Suites](suites.md)
- [k6 Harness](k6.md)
- [Perf Regression Policy](../observability/perf-regression-policy.md)
- `.github/workflows/perf-nightly.yml`
- `ops-ci`
