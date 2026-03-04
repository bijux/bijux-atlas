---
title: Ops artifact consumption
audience: operators
type: guide
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - release
related:
  - docs/reference/release-bundle-index.md
---

# Ops artifact consumption

## Helm chart

```bash
helm pull oci://ghcr.io/bijux/charts/bijux-atlas --version 0.1.0
helm install atlas ./bijux-atlas-0.1.0.tgz -f ops/k8s/values/prod.yaml
```

## Profiles bundle

Download the release bundle artifact and unpack:

```bash
tar -xzf profiles-bundle-0.1.0.tar.gz
```

Then use:

- `install-matrix.json`
- values files under `values/`
- `values.schema.json`
- `cluster-prerequisites.json`
