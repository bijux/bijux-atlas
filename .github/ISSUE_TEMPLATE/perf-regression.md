---
name: Performance regression
about: Report a load/perf regression detected by CI or nightly suites
title: "perf: regression in <suite-name>"
labels: [performance, regression]
assignees: ''
---

## What

Describe the regression and affected suite(s).

## Evidence

- Suite name(s):
- Run URL:
- Commit SHA:
- Image digest:
- Dataset hash:
- Policy hash:

## Contracts

- Threshold source: `configs/perf/k6-thresholds.v1.json`
- Suite source: `ops/load/suites/suites.json`
- Score output: `artifacts/ops/e2e/k6/score.md`

## Failure modes

- p95/p99 drift
- error-rate regression
- cold-start regression
- soak memory growth

## How to verify

```bash
make ops-load-nightly
make ops-load-run
```

Expected output shape: violating suite names and threshold deltas.

## See also

- `docs/operations/runbooks/load-failure-triage.md`
- `docs/operations/observability/perf-regression-policy.md`
