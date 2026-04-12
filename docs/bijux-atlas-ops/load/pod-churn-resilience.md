---
title: Pod Churn Resilience
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Pod Churn Resilience

Pod churn scenarios verify whether Atlas stays healthy when runtime instances
are killed or rotated during live traffic.

## Purpose

Use this scenario when validating that Atlas can survive pod loss, restart, or
rotation without turning routine churn into an outage.

## Source of Truth

- `ops/load/scenarios/pod-churn.json`
- `ops/load/suites/suites.json`
- `ops/load/contracts/k6-thresholds.v1.json`

## What Pod Churn Means

For Atlas, pod churn includes:

- intentional rollout or restart replacement
- unexpected pod loss during live traffic
- repeated rotation that forces readiness and connection recovery

The current scenario definition requires Kubernetes and runs the
`warm-steady.js` load shape while pods are killed or rotated.

## What Healthy Recovery Looks Like

- latency rises only within the declared churn thresholds
- error rate remains bounded rather than cascading
- readiness recovers quickly enough that service continuity holds
- observability signals make the churn visible to operators

## Transient Turbulence Versus Regression

Short-lived latency movement is expected during pod churn. It becomes a release
regression when:

- `p95` or `p99` exceeds the pod-churn thresholds
- error rate moves beyond the allowed budget
- readiness does not recover or traffic black-holes during replacement

## Related Contracts and Assets

- `ops/load/scenarios/pod-churn.json`
- `ops/load/contracts/k6-thresholds.v1.json`
