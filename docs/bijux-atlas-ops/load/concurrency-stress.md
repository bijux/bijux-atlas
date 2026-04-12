---
title: Concurrency Stress
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Concurrency Stress

Concurrency stress scenarios validate saturation behavior and the quality of
Atlas limit enforcement under pressure.

## Purpose

Use these scenarios to distinguish normal concurrency scaling from saturation,
queueing, or overload behavior that should block rollout or capacity claims.

## Source of Truth

- `ops/load/generated/concurrency-stress-scenarios.json`
- `ops/load/scenarios/`
- `ops/load/contracts/k6-thresholds.v1.json`

## Scenario Taxonomy

The generated concurrency registry currently defines three scenario shapes:

- `load-single-client-baseline` for a low-contention reference point
- `load-multi-client-concurrency` for realistic concurrent traffic
- `load-saturation-stress` for pressure near or beyond the intended runtime
  limit

## What These Scenarios Validate

- whether concurrency limits are enforced rather than bypassed
- whether saturation raises bounded latency instead of silent correctness drift
- whether queueing, shedding, or overload signals appear when expected
- whether cheap or protected traffic classes stay available under pressure

## Metrics That Matter

Track at least:

- request latency percentiles
- request failure rate
- throughput under concurrent pressure
- overload or queue-depth signals when the scenario is intended to saturate the
  system

Healthy limit enforcement means Atlas makes the pressure visible and keeps
responses within declared degradation policy. Saturation becomes a failure when
latency, error rate, or overload behavior moves beyond the threshold contract.

## Related Contracts and Assets

- `ops/load/generated/concurrency-stress-scenarios.json`
- `ops/load/contracts/k6-thresholds.v1.json`
- `ops/load/scenarios/`
