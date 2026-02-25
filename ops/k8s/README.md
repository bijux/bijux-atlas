# Ops K8s

## Purpose
Own Helm chart delivery, install profiles, and Kubernetes-only validation gates.

## Entry points
- `make ops-deploy PROFILE=kind`
- `make ops-undeploy PROFILE=kind`
- `make ops-k8s-suite PROFILE=kind`
- `make ops-k8s-contracts`

## Contracts
- `ops/k8s/CONTRACT.md`
- `ops/schema/k8s/install-matrix.schema.json`
- `ops/schema/k8s/inventory-index.schema.json`
- `ops/schema/k8s/render-artifact-index.schema.json`
- `ops/schema/k8s/release-snapshot.schema.json`

## Artifacts
- `ops/_artifacts/<run_id>/k8s/`
- `ops/k8s/generated/inventory-index.json`
- `ops/k8s/generated/render-artifact-index.json`
- `ops/k8s/generated/release-snapshot.json`

## Failure modes
- Chart/value schema mismatch.
- Install matrix drift from declared profiles.
- Cluster policy regression in test suite.
