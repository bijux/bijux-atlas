---
title: Ops as product
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Ops as product

Ops distribution ships a consumable surface, not only repository source.

## Shipped artifacts

- OCI chart: `oci://ghcr.io/bijux/charts/bijux-atlas`
- Offline bundle: `artifacts/release/ops/bundle/vX.Y.Z/ops-bundle-vX.Y.Z.tar.gz`
- Chart digest manifest: `release/ops-chart-digest.json`
- Ops release manifest: `release/ops-release-manifest.json`
- Evidence bundles: `release/evidence/ops-distribution/*/evidence.json`

## Why this surface exists

- Operators need reproducible install and rollback paths.
- Auditors need deterministic manifest and digest lineage.
- Support needs evidence bundles with cluster, helm, chart, image, and profile metadata.
