---
title: Load
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Load

`bijux-atlas-ops/load` explains the Atlas load-testing system as an operations
control surface, not just a directory of k6 files.

## Purpose

Use this section to understand how scenarios, suites, thresholds, baselines, CI
lanes, generated summaries, and regression contracts fit together before a
change is promoted.

## Source of Truth

- `ops/load/scenarios/` defines named scenario intent
- `ops/load/suites/suites.json` defines suite taxonomy, expected metrics, and
  must-pass status
- `ops/load/thresholds/` and `ops/load/contracts/k6-thresholds.v1.json` define
  scenario-specific and shared budgets
- `ops/load/baselines/` defines committed baseline evidence
- `ops/load/k6/` and `ops/load/k6/suites/` define executable load generators
- `ops/load/generated/` defines generated summaries and coverage outputs
- `ops/load/ci/load-harness-ci-scenario.json` and the workflow lanes define CI
  enforcement paths

## Load System Shape

Atlas load validation works in layers:

1. Scenarios define the question being asked.
2. Suites collect those scenarios into operational lanes such as smoke, nightly,
   pull-request, and broader load CI.
3. Thresholds and budgets decide what counts as acceptable behavior.
4. Baselines capture the approved reference point for comparison.
5. Generated summaries and CI contracts turn the run into reviewable evidence.

## Operator Workflow

1. Pick the scenario family that matches the change risk.
2. Confirm the suite, thresholds, and expected metrics for that family.
3. Run the workload in the right lane or environment.
4. Compare results against the approved baseline.
5. Carry the generated evidence into rollout or release review.

## Pages

- [Baseline Management](baseline-management.md)
- [Benchmark CI](benchmark-ci.md)
- [Concurrency Stress](concurrency-stress.md)
- [Failure Injection Load](failure-injection-load.md)
- [Load Suites](load-suites.md)
- [Performance and Load](performance-and-load.md)
- [Pod Churn Resilience](pod-churn-resilience.md)
- [Rollout Under Load](rollout-under-load.md)
- [Scenario Registry](scenario-registry.md)
- [Thresholds and Budgets](thresholds-and-budgets.md)
