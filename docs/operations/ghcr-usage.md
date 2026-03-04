---
title: GHCR usage
audience: operators
type: guide
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/reference/docker.md
---

# GHCR usage

## Pull images

```bash
docker pull ghcr.io/bijux/bijux-atlas-runtime:v0.1.0
docker pull ghcr.io/bijux/bijux-atlas-runtime:sha-<short-sha>
```

## Run image

```bash
docker run --rm -p 8080:8080 ghcr.io/bijux/bijux-atlas-runtime:v0.1.0 version
```

## Artifact expectations

Each published image release provides:

- SPDX and CycloneDX SBOMs
- vulnerability scan report
- image digest report
- provenance bundle
