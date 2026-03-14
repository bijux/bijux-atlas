---
title: Installing Ops Product
audience: operator
type: guide
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-05
tags:
  - operations
  - release
  - helm
related:
  - docs/operations/ops-distribution-policy.md
  - docs/operations/ops-provenance.md
  - docs/operations/release-artifacts-overview.md
  - docs/operations/ops-compatibility-matrix.md
---

# Installing Ops Product

Ops publication uses OCI Helm charts in GHCR.

## Publication decision

- Mechanism: OCI Helm chart
- Registry: `ghcr.io`
- Chart reference: `oci://ghcr.io/bijux/charts/bijux-atlas`
- Chart name: `bijux-atlas`

## Version and provenance policy

- Chart tags are release versions from `ops/release/ops-v0.1.toml`.
- Consumers must pin by digest when moving to production.
- Provenance binding is recorded in:
  - `ops/release/ops-chart-digest.json`
  - `ops/release/ops-release-manifest.json`
  - `ops/release/ops-release-bundle-manifest.json`

## Install flow

```bash
helm pull oci://ghcr.io/bijux/charts/bijux-atlas --version 0.1.0
helm install atlas ./bijux-atlas-0.1.0.tgz -f ops/k8s/values/prod.yaml
```

For offline bundle installation, see [Installing Ops Bundle](installing-ops-bundle.md).

## Verify

1. Run `bijux-dev-atlas release ops provenance verify --format json`
2. Confirm digest match between pulled chart and `ops/release/ops-chart-digest.json`
3. Confirm version linkage in `ops/release/ops-release-bundle-manifest.json`
4. Confirm compatibility row with `bijux-dev-atlas release ops compatibility-matrix --format json`
