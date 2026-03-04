---
title: Ops distribution policy
audience: operators
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - release
related:
  - docs/operations/ops-artifact-consumption.md
---

# Ops distribution policy

Ops release artifacts are distributed through:

- OCI Helm chart publication (`ghcr.io/<owner>/charts/bijux-atlas`)
- versioned profiles bundle archive

Policy source of truth: `configs/release/ops-distribution-policy.json`.
