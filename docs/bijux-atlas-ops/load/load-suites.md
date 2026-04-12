---
title: Load Suites
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Load Suites

Load suites under `ops/load/suites/` define the named workload families Atlas
uses for operational verification.

## Purpose

Use the load suites registry to understand which workload families are must-pass,
which lanes they run in, and which metrics and thresholds govern the result.

## Source of Truth

- `ops/load/suites/suites.json`
- `ops/load/k6/suites/`
- `ops/load/contracts/suite-schema.json`
- `ops/load/generated/load-summary.json`

## Suite Taxonomy

`ops/load/suites/suites.json` defines suite entries with:

- a `name`
- a `purpose`
- a `kind`, such as `k6` or a specialized script runner
- a scenario or runner binding
- `must_pass` status
- the CI or review lanes in `run_in`
- expected metrics and inline threshold expectations

Representative suite families include:

- baseline confidence checks such as `mixed` and `cheap-only-survival`
- latency and warmup checks such as `warm-steady-state-p99`,
  `cold-start-p99`, and `cold-start-prefetch-5pods`
- resilience checks such as `stampede`, `store-outage-under-spike`,
  `noisy-neighbor-cpu-throttle`, and `pod-churn`
- workload-specific families such as `mixed-workload`,
  `ingest-query-workload`, `heavy-query-workload`, `read-heavy-workload`, and
  `write-heavy-workload`

## Evidence Produced

Suite execution should produce:

- the selected suite list
- scenario coverage against `ops/load/generated/load-summary.json`
- the expected metrics for each suite
- the pass or fail result relative to thresholds
