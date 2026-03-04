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

## Tagging conventions

- Release tags: `vX.Y.Z`
- Immutable build tags: `sha-<short-sha>`
- Optional moving tag (if enabled): `latest`

## Metadata requirements

Images should carry OCI labels for source, revision, version, and license.
