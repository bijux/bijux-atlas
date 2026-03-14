---
title: Release bundle index
audience: operators
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - release
related:
  - docs/reference/release-plan.md
---

# Release bundle index

## Docs

- GitHub Pages site artifact: `artifacts/docs/site`

## Containers

- Runtime image: `ghcr.io/<owner>/bijux-atlas-runtime:vX.Y.Z`
- Immutable image tag: `ghcr.io/<owner>/bijux-atlas-runtime:sha-<short>`
- SBOM and scan artifacts: `artifacts/docker-publish/*`

## Ops

- OCI Helm chart: `oci://ghcr.io/<owner>/charts/bijux-atlas`
- Profiles bundle tarball: `artifacts/ops-publish/profiles-bundle-<version>.tar.gz`
- Ops bundle manifest: `artifacts/ops-publish/ops-bundle-manifest.json`
- Ops install matrix report: `artifacts/ops-publish/install-matrix-report.json`
- Ops artifact consumption guide: `docs/operations/ops-artifact-consumption.md`
- Ops offline install guide: `docs/operations/ops-offline-install-guide.md`

## Release evidence

- Release manifest: `ops/release/release-manifest.json`
- Evidence manifest: `ops/release/evidence/manifest.json`
- Signing and provenance: `ops/release/signing/*.json`, `ops/release/provenance.json`
