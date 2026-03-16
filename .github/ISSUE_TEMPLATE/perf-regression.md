---
name: Performance regression
about: Report a performance or load regression detected by CI, nightly, or manual verification
title: "perf: regression in <suite-or-surface>"
labels: [performance, regression]
assignees: ''
---

## What

Describe the regression, the affected surface, and the expected behavior.

## Evidence

- Detection source:
- Suite name(s):
- Run URL:
- Commit SHA:
- Baseline run or artifact:
- Image digest:
- Dataset hash:
- Policy hash:

## Contract Sources

- Threshold policy: `configs/sources/operations/perf/k6-thresholds.v1.json`
- Load thresholds contract: `ops/load/contracts/k6-thresholds.v1.json`
- Suite source: `ops/load/suites/suites.json`
- Load manifest: `ops/load/load.toml`

## Failure modes

- p95/p99 drift
- error-rate regression
- cold-start regression
- soak memory growth

## How to verify

```bash
make ops-load-plan SUITE=mixed
make ops-load-run SUITE=mixed
```

Expected output shape: violating suite names, threshold deltas, and emitted load report artifacts.

## See also

- `docs/04-operations/incident-response.md`
- `docs/04-operations/performance-and-load.md`
- `docs/06-development/automation-control-plane.md`
