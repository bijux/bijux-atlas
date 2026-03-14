---
title: Image security update policy
audience: operators
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - security
related:
  - docs/operations/supply-chain-policies.md
  - docs/operations/release-signing.md
---

# Image security update policy

- Base image digests are pinned and reviewed through `ops/docker/bases.lock`.
- HIGH and CRITICAL vulnerabilities require explicit allowlist justification under governed Docker policy checks.
- Release publication requires SBOM and vulnerability scan artifacts.
- Security updates must preserve reproducible build metadata and provenance artifacts.

## Supported platforms

Runtime image publication targets:

- `linux/amd64`
- `linux/arm64`


Enforcement reference: OPS-ROOT-023.
