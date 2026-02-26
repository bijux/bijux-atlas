# Ops K8s

- Owner: `bijux-atlas-operations`
- Purpose: `kubernetes delivery contracts and render/install validation surfaces`
- Consumers: `bijux dev atlas ops k8s commands, checks_ops_domain_contract_structure`

## Purpose
Own Helm chart delivery, install profiles, and Kubernetes-only validation gates.

## Entry points
- `make ops-k8s-suite PROFILE=kind`
- `make ops-k8s-contracts`
- `make ops-k8s-tests PROFILE=kind`
- `make ops-k8s-template-tests PROFILE=kind`
- `bijux dev atlas ops k8s apply --profile kind --apply --allow-subprocess --allow-write`

## Contracts
- `ops/k8s/CONTRACT.md`
- `ops/schema/k8s/install-matrix.schema.json`
- `ops/schema/k8s/inventory-index.schema.json`
- `ops/schema/k8s/render-artifact-index.schema.json`
- `ops/schema/k8s/release-snapshot.schema.json`

## Artifacts
- `artifacts/atlas-dev/ops/<run_id>/k8s/`
- `ops/k8s/generated/inventory-index.json`
- `ops/k8s/generated/render-artifact-index.json`
- `ops/k8s/generated/release-snapshot.json`

## Failure modes
- Chart/value schema mismatch.
- Install matrix drift from declared profiles.
- Cluster policy regression in test suite.

Placeholder extension directory tracked with `.gitkeep`: `ops/k8s/tests/checks/perf`.
