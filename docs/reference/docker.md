---
title: Docker reference
audience: operators
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - docker
related:
  - docs/reference/index.md
  - docs/operations/deploy.md
---

# Docker reference

## Image surfaces

- `docker/atlas-runtime.Dockerfile`: runtime image for service execution.
- `docker/atlas-dev.Dockerfile`: development and validation image surface.
- `docker/images/runtime/runtime-config.example.yaml`: minimal runtime config example.

## Tagging conventions

- Release tags: `vX.Y.Z`
- Immutable build tags: `sha-<short-sha>`
- Optional moving tag (if enabled): `latest`

## Supported platforms

- `linux/amd64`
- `linux/arm64`

## Metadata requirements

Images should carry OCI labels for source, revision, version, and license.

## Policy and runtime references

- [Runtime image contract](../operations/runtime-image-contract.md)
- [Image security update policy](../operations/image-security-update-policy.md)
- [Docker build network policy](../operations/docker-build-network-policy.md)
- [GHCR usage](../operations/ghcr-usage.md)
- [Image compatibility policy](../operations/image-compatibility-policy.md)
- [Image tag deprecation policy](../operations/image-tag-deprecation-policy.md)
- [Local docker build policy](../operations/local-docker-build-policy.md)
