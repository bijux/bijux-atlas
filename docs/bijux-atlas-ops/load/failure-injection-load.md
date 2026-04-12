---
title: Failure Injection Load
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Failure Injection Load

Atlas combines failure injection and load scenarios so resilience claims are
measured under degraded conditions, not only happy-path traffic.

## Purpose

Use this page when validating graceful degradation, correctness preservation, or
recovery behavior while Atlas is under meaningful traffic.

## Source of Truth

- `ops/e2e/scenarios/failure/`
- `ops/load/scenarios/`
- `ops/load/thresholds/`

## Combined Resilience Model

The failure program defines injections such as invalid config, missing
artifacts, corrupted shards, disk exhaustion, ingest crashes, query crashes, bad
request floods, and slow-query warning conditions. The load program pairs those
ideas with traffic scenarios such as:

- `store-outage-under-spike`
- `noisy-neighbor-cpu-throttle`
- `pod-churn`
- `stampede`
- `cheap-only-survival`

## What Operators Are Testing

Operators should state the hypothesis before running the scenario:

- graceful degradation: protected traffic classes stay available and Atlas
  reports overload honestly
- correctness preservation: degraded conditions do not return wrong data or
  break contract semantics
- recovery: the service stabilizes after the injected failure is removed

## How to Judge the Outcome

- graceful degradation means Atlas may slow down or shed selected traffic, but
  the protected surface stays within the declared thresholds
- correctness failure means the service returns invalid, inconsistent, or
  contract-breaking results even if latency looks acceptable
- resilience failure means the service does not recover or the incident surface
  becomes opaque to operators

## Related Contracts and Assets

- `ops/e2e/scenarios/failure/`
- `ops/load/scenarios/`
- `ops/load/thresholds/`
