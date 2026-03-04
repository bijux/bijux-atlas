# Operator Quickstart

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: give operators the minimum path for a baseline-profile deployment.

## Prereqs

- Access to the Kubernetes cluster and namespace.
- Helm and kubectl configured for the target cluster.

## Install

```bash
helm lint ops/k8s/charts/bijux-atlas -f ops/k8s/values/profile-baseline.yaml
helm upgrade --install bijux-atlas ops/k8s/charts/bijux-atlas -f ops/k8s/values/profile-baseline.yaml
```

## Verify

- Confirm pods become ready.
- Confirm `/healthz` and `/readyz` respond as expected.
- Confirm the deployed values still match the baseline intent.

## Rollback

Use the last known-good release revision or reapply the last known-good baseline overlay.
