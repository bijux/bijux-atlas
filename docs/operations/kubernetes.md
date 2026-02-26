# Kubernetes Operations

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## What

Canonical Kubernetes operations flow and contract-aligned make entrypoints.

## Must-Pass Local Flow

`make ops-full` must pass this chain:

1. `ops-up`
2. `ops-deploy`
3. `ops-publish`
4. `ops-warm`
5. `ops-smoke`
6. `ops-k8s-tests`
7. `ops-load-smoke`
8. `ops-observability-validate`

## Chart Contracts

- Required resources/limits.
- Required readiness/liveness probes.
- Perf profile sets `readinessProbePath: /healthz/overload` so readiness follows real admission-control state during load.
- Required `PodDisruptionBudget`.
- HPA behavior validated via k8s tests.
- NetworkPolicy behavior validated via k8s tests.
- ServiceMonitor behavior is optional and tested for CRD present/absent paths.

## Scale, Rollout, and Rollback

- Scale checks: `make ops-k8s-tests` (`test_hpa.sh`).
- Rollout under load drill: `make ops-drill-upgrade-under-load`.
- Rollback under load drill: `make ops-drill-rollback-under-load`.
- Pod churn drill: `make ops-drill-pod-churn`.
- Store outage drill: `make ops-drill-store-outage`.

## Node-Local Cache Profile

- Profile and contract: `docs/operations/k8s/node-local-shared-cache-profile.md`.
- Validation: `make ops-k8s-tests` (`test_node_local_cache_profile.sh`, `test_storage_modes.sh`).

## Failure Bundles

- `ops-full`, `ops-full-pr`, and `ops-full-nightly` install failure traps.
- On failure, bundles include logs, events, manifests, and performance outputs under `artifacts/ops/`.
- `OPS_MODE=full make ops-full` additionally enforces churn/outage/rollback-under-load drills.

## SLO Validation

- Smoke and observability flows validate latency/error metrics contracts:
  - `make ops-smoke`
  - `make ops-observability-validate`
