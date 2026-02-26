# CRD Dependency Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines local-cluster CRD expectations for chart features.

## Why

Keeps local tests runnable without mandatory external CRD installers.

## Contracts

- Local profile disables ServiceMonitor by default (`ops/k8s/values/local.yaml`).
- ServiceMonitor CRD is optional for local development.
- Tests must pass without CRD installation when `serviceMonitor.enabled=false`.

## Failure modes

Hard CRD requirements break local installation and CI portability.

## How to verify

```bash
$ make ops-k8s-template-tests
$ make ops-k8s-tests
```

Expected output: chart renders and installs in local profile without CRD blockers.

## See also

- [Helm Chart Contract](chart.md)
- [K8s Test Contract](k8s-test-contract.md)
- `ops-k8s-tests`

- Chart values anchor: `values.server`
