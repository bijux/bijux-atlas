---
title: Upgrade Validation
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Upgrade Validation

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@3af24f78bdf0be1507efa8651298c45b68fa9e1e`
- Last changed: `2026-03-03`
- Reason to exist: define the kind-backed lifecycle validation path for upgrade and rollback simulations.

## Prereqs

- Create or reuse the kind simulation cluster with `bijux dev atlas ops kind up`.
- Keep the previous chart package at `artifacts/ops/chart-sources/previous/bijux-atlas.tgz`.
- Use a governed profile such as `profile-baseline`, `ci`, `offline`, or `perf`.

## Install

```bash
bijux dev atlas ops helm install --profile profile-baseline --cluster kind --chart-source previous --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops helm upgrade --profile profile-baseline --cluster kind --to current --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops helm rollback --profile profile-baseline --cluster kind --to previous --allow-subprocess --allow-write --allow-network --format json
```

## Verify

- `ops-upgrade.json` must record a successful readiness wait, successful smoke checks, rollout history, pod restart count, and passing compatibility checks.
- `ops-rollback.json` must record a successful readiness wait, successful smoke checks, and `service_healthy_after_rollback=true`.
- `ops-lifecycle-summary.json` must contain the profile entry with stable report paths for the upgrade and rollback evidence.

## Rollback

- If `ops-upgrade.json` fails, do not continue to new values or a new chart package.
- Use `ops helm rollback --to previous` and confirm `ops-rollback.json` returns `status=ok` before leaving the environment in service.
