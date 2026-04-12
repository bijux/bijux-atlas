---
title: Rollout Under Load
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Rollout Under Load

Atlas explicitly tests steady load during rollout activity so release safety is
measured under realistic pressure.

## Purpose

Use these scenarios when a deployment change is risky enough that normal steady
state validation is not sufficient. The goal is to prove the service can change
version while still serving meaningful traffic.

## Source of Truth

- `ops/load/scenarios/load-under-rollout.json`
- `ops/load/scenarios/load-under-rollback.json`
- `ops/load/contracts/k6-thresholds.v1.json`
- `docs/bijux-atlas-ops/kubernetes/rollout-safety.md`

## Test Model

The rollout-under-load program has four distinct stages:

1. establish steady traffic with the governed workload shape
2. trigger the rollout or rollback event
3. observe latency, error behavior, and readiness during the transition
4. confirm the service stabilizes within the declared thresholds

Both current scenario definitions require Kubernetes and use the
`warm-steady.js` traffic shape, which keeps the comparison anchored to a known
steady-state workload.

## Acceptance Rules

- latency and failure rate must stay within the rollout-specific thresholds
- readiness transitions must stay aligned with rollout safety expectations
- rollback must restore service behavior rather than just complete a control
  action

## Related Contracts and Assets

- `ops/load/scenarios/load-under-rollout.json`
- `ops/load/scenarios/load-under-rollback.json`
- `ops/load/contracts/k6-thresholds.v1.json`
