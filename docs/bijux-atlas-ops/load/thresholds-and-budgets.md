---
title: Thresholds and Budgets
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Thresholds and Budgets

Load thresholds, failure budgets, and scenario expectations are stored as
reviewable data instead of implicit dashboard memory.

## Purpose

Use this page to understand how Atlas classifies acceptable latency, error,
saturation, survival, and degradation behavior across its load scenarios.

## Source of Truth

- `ops/load/thresholds/`
- `ops/load/contracts/k6-thresholds.v1.json`
- `ops/load/contracts/performance-regression-thresholds.json`

## Threshold Classes

Atlas thresholds fall into five operational classes:

- latency thresholds such as `p95_ms` and `p99_ms`
- error-rate thresholds such as `fail_rate`
- saturation thresholds for CPU, disk, thread pools, and queue pressure
- survival thresholds for cheap-path or degraded-mode availability
- degradation thresholds for scenarios like rollout under load, pod churn, and
  store outage

## Threshold Relationships

- `ops/load/contracts/k6-thresholds.v1.json` defines the shared scenario-level
  defaults that many suites inherit
- `ops/load/thresholds/*.thresholds.json` holds per-scenario files for the
  operational surface under review
- `ops/load/contracts/performance-regression-thresholds.json` defines the
  allowed delta between an approved baseline and a candidate run

## How Operators Should Use Them

1. start from the scenario-specific threshold file when it exists
2. cross-check the matching values in `k6-thresholds.v1.json`
3. compare candidate results to the approved baseline and the regression
   percentage thresholds
4. escalate any change that claims success while only meeting a weaker local
   threshold set

## Related Contracts and Assets

- `ops/load/thresholds/`
- `ops/load/contracts/k6-thresholds.v1.json`
- `ops/load/contracts/performance-regression-thresholds.json`
