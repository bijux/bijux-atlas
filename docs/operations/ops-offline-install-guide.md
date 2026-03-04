---
title: Ops offline install guide
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - offline
related:
  - docs/operations/ops-artifact-consumption.md
---

# Ops offline install guide

Prefetch and transfer these artifacts:

- OCI chart package archive
- profiles bundle archive
- image digest report
- SBOM and scan artifacts for audit

Use `values/offline.yaml` from the profiles bundle and preloaded container images in the target registry.
