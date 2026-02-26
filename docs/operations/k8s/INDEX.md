# Kubernetes Operations Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-server`

This is the canonical index for Kubernetes operations docs.

## Topics

- Chart contract: `chart.md`
- K8s test contract: `k8s-test-contract.md`
- Network policy correctness: `network-policy-correctness.md`
- Cluster resource profile: `cluster-resource-profile.md`
- RBAC minimalism: `rbac-minimalism.md`
- CRD dependency policy: `crd-dependency-policy.md`
- Kustomize policy: `kustomize-policy.md`
- Release install matrix: `release-install-matrix.md`
- Packaging and operational profile: `packaging-and-ops.md`
- Values schema and deployment constraints: `values-schema.md`
- Autoscaling contract: `autoscaling-contract.md`
- Plugin-mode entrypoint: `plugin-mode-entrypoint.md`
- Progressive delivery: `canary-progressive-delivery.md`
- Node locality and shard scaling: `dataset-locality-and-shard-scaling.md`
- Node-local shared cache profile: `node-local-shared-cache-profile.md`
- Warm on rollout hook: `warm-on-rollout.md`
- Layer drift triage: `when-layer-drift-fails.md`

## What

Section overview.

## Why

Rationale for this section.

## Scope

Scope of documents in this section.

## Non-goals

Out-of-scope topics for this section.

## Contracts

Normative links and rules.

## Failure modes

Failure modes if docs drift.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-ci`
