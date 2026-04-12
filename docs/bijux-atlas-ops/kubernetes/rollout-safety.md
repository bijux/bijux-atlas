---
title: Rollout Safety
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Rollout Safety

Rollout safety is documented as an explicit contract over values files,
release flows, and supporting suites.

## Purpose

Use this page to decide whether a profile is allowed to deploy as a plain
deployment or a governed rollout, which readiness gates must hold, and when the
operator must stop and roll back.

## Source of Truth

- `ops/k8s/rollout-safety-contract.json`
- `ops/schema/k8s/rollout-safety-contract.schema.json`
- `ops/k8s/install-matrix.json`
- `ops/e2e/scenarios/upgrade/`

## What Is Governed

The rollout safety contract currently defines profile-level invariants such as:

- `rollout_mode`, which separates simple deployment paths from governed rollout
  paths
- `warmup_required`, which determines whether startup data preparation must
  finish before promotion
- `readiness_path_required`, which keeps readiness endpoints mandatory for every
  supported profile
- `requiredToggles` and `forbiddenToggles`, which prevent risky runtime and
  profile combinations
- `networkPolicyModeRequired` and `hpaPolicyRequired`, which tie rollout safety
  to isolation and scaling posture

## Rollout Invariants

Treat these as non-negotiable rules before promotion:

- readiness must reflect actual serving ability for the selected profile
- drain and termination settings must allow in-flight requests to complete
- warmup jobs must complete when the profile requires them
- production-oriented rollout paths must keep their declared network and scaling
  policies intact
- rollback references must be explicit for upgrade and rollback scenarios

## Rollback Triggers

Start rollback review immediately when any of these happen:

- readiness never stabilizes after the allowed rollout window
- validation or conformance evidence shows broken probes, missing policy, or
  failed rollout groups
- latency or error behavior regresses during the rollout-under-load path
- the running profile no longer satisfies its required toggles

## How to Validate

1. Confirm the target profile entry in `ops/k8s/rollout-safety-contract.json`.
2. Check the matching install or upgrade scenario in
   `ops/k8s/install-matrix.json`.
3. Run render, validate, and the required suite for that profile.
4. Review the related upgrade or rollback scenario assets under
   `ops/e2e/scenarios/upgrade/`.
5. Carry observability and load evidence into the promotion decision for
   rollout-based profiles.

## Evidence Produced

Rollout safety review should be backed by:

- render and validate reports for the exact profile
- conformance suite evidence for the selected rollout path
- upgrade or rollback scenario references
- readiness, drain, and load evidence when the rollout mode is not a simple
  deployment

## Related Contracts and Assets

- `ops/schema/k8s/rollout-safety-contract.schema.json`
- `ops/e2e/scenarios/upgrade/`
- `ops/k8s/rollout-safety-contract.json`
- `ops/k8s/install-matrix.json`
