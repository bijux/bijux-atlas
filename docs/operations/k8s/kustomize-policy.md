# Kustomize Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines packaging policy for Helm-only deployment surface.

## Why

Avoids duplicate packaging systems and drift between overlays and charts.

## Contracts

- Helm is the only supported packaging path for atlas k8s deployment.
- Kustomize overlays are not maintained in this repository.

## Failure modes

Multiple packaging systems create inconsistent runtime behavior.

## How to verify

```bash
$ make ops-k8s-template-tests
```

Expected output: Helm templates render without requiring kustomize overlays.

## See also

- [Helm Chart Contract](chart.md)
- [K8s Test Contract](k8s-test-contract.md)
- `ops-k8s-template-tests`

- Chart values anchor: `values.server`
