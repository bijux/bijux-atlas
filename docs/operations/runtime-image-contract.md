---
title: Runtime image contract
audience: operators
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/reference/docker.md
  - docs/operations/release-evidence.md
---

# Runtime image contract

## Included

- Runtime binary `/app/bijux-atlas`
- OCI metadata labels for source, version, revision, created, license
- Distroless runtime base image with digest pinning

## Excluded

- Build toolchains in runtime layer
- Package managers and shell utilities
- Mutable runtime dependency installation

## Runtime config example

`docker/images/runtime/runtime-config.example.yaml` is the canonical minimal config sample shipped with the image source.

## Healthcheck stance

The runtime image intentionally does not use Dockerfile `HEALTHCHECK` because the distroless base omits shell tooling. Health validation is performed through HTTP `/healthz` and `/readyz` endpoints from orchestration probes.
