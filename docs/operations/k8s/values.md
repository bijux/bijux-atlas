# Kubernetes Values

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Generated summary of Helm top-level values from the chart-values contract.

## Why

Keeps operations docs aligned to chart values SSOT.

## Scope

Top-level chart values keys only.

## Non-goals

Does not redefine schema semantics beyond contract references.

## Contracts
- `values.affinity`
- `values.alertRules`
- `values.cache`
- `values.catalog`
- `values.catalogPublishJob`
- `values.concurrency`
- `values.datasetWarmupJob`
- `values.extraEnv`
- `values.hpa`
- `values.image`
- `values.ingress`
- `values.metrics`
- `values.networkPolicy`
- `values.nodeLocalSsdProfile`
- `values.nodeSelector`
- `values.pdb`
- `values.podSecurityContext`
- `values.priorityClassName`
- `values.rateLimits`
- `values.replicaCount`
- `values.resources`
- `values.rollout`
- `values.securityContext`
- `values.sequenceRateLimits`
- `values.server`
- `values.service`
- `values.serviceMonitor`
- `values.store`
- `values.terminationGracePeriodSeconds`
- `values.tolerations`

## Failure modes

Missing or stale keys can break deployments and profile docs.

## How to verify

```bash
$ make ops-values-validate
```

Expected output: generated values doc and chart contract check pass.

## See also

- [Chart Values Contract](../../contracts/chart-values.md)
- [Helm Chart Contract](chart.md)
- [K8s Index](INDEX.md)
- `ops-values-validate`
