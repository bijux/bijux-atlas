---
title: Baseline Management
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Baseline Management

Performance baselines are reviewed artifacts, not vague memories of how the
system felt during a previous run.

## Purpose

Use this page to understand when a baseline may be created or refreshed, who is
allowed to change it, and how drift is evaluated before it becomes the new
reference.

## Source of Truth

- `ops/load/baselines/ci-runner.json`
- `ops/load/baselines/local.json`
- `ops/load/baselines/system-load-baseline.json`
- `ops/load/contracts/load-summary.schema.json`
- `ops/load/contracts/performance-regression-thresholds.json`

## Baseline Lifecycle

Atlas currently keeps baseline evidence for CI, local validation, and broader
system-load review:

- `ci-runner.json` is the deterministic CI reference for fast regression checks
- `local.json` is the local operator reference with the same smoke-aligned
  suites
- `system-load-baseline.json` covers the broader workload families used for
  system-level performance review

Each baseline records metadata such as capture time, profile, and tool versions
so operators can explain what environment produced the approved numbers.

## Update Rules

Only update a baseline when:

- the run is reproducible in the intended profile
- the changed scenario family still maps to the declared suites and thresholds
- the comparison shows intentional movement rather than unexplained drift
- the review includes evidence for why the new numbers are safer or more
  accurate

## Drift Review

Review baseline drift in two buckets:

- acceptable drift: an explained change in workload behavior that stays within
  the regression contract and is approved as the new normal
- blocking drift: unexplained latency, throughput, or error movement that
  exceeds the thresholds in
  `ops/load/contracts/performance-regression-thresholds.json`

## Evidence Produced

Baseline updates should come with:

- the committed baseline file update
- the candidate run output used for comparison
- the regression comparison summary
- notes on profile, dataset tier, and tool version changes

## Related Contracts and Assets

- `ops/load/baselines/ci-runner.json`
- `ops/load/baselines/local.json`
- `ops/load/baselines/system-load-baseline.json`
- `ops/load/contracts/performance-regression-thresholds.json`
