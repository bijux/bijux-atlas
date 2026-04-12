---
title: Benchmark CI
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Benchmark CI

Atlas uses dedicated GitHub workflow lanes for ingest, query, load, and
performance regression checks.

## Purpose

Use this page to understand which CI lane produces which benchmark evidence and
which lanes are promotion gates versus informational signals.

## Source of Truth

- `.github/workflows/load-system-ci.yml`
- `.github/workflows/performance-regression-ci.yml`
- `.github/workflows/ingest-benchmark-ci.yml`
- `.github/workflows/query-benchmark-ci.yml`
- `ops/load/ci/load-harness-ci-scenario.json`
- `ops/load/contracts/performance-regression-ci-contract.json`

## Lane Model

The benchmark lanes serve different operator needs:

- `load-system-ci.yml` runs system-load validation and is the broadest lane for
  sustained workload review
- `performance-regression-ci.yml` enforces regression comparison rules and is a
  gating lane when baseline deltas exceed the contract
- `ingest-benchmark-ci.yml` focuses on ingest-adjacent performance surfaces
- `query-benchmark-ci.yml` focuses on query behavior and request-shape pressure

`ops/load/ci/load-harness-ci-scenario.json` defines the core CI harness flow:
run baseline, run candidate, compare, and fail with exit code `2` on
regression.

## Trigger and Output Expectations

The benchmark CI program should always make these outputs reviewable:

- the scenario or suite that ran
- the raw run report in machine-readable form
- the comparison against the approved baseline
- a clear pass or fail meaning tied to the regression contract

## Gates Versus Informational Lanes

- regression-comparison lanes are gates when they enforce the required baseline,
  run, and compare flow
- broader benchmark lanes may be informational when they provide trend or
  exploratory performance data without blocking promotion on their own
- operators should not treat an informational trend lane as a substitute for a
  required regression gate

## Related Contracts and Assets

- `.github/workflows/load-system-ci.yml`
- `.github/workflows/performance-regression-ci.yml`
- `.github/workflows/ingest-benchmark-ci.yml`
- `.github/workflows/query-benchmark-ci.yml`
- `ops/load/ci/load-harness-ci-scenario.json`
- `ops/load/contracts/performance-regression-ci-contract.json`
