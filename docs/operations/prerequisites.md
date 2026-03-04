---
title: Operations Prerequisites
audience: operator
type: reference
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - deployment
  - prerequisites
related:
  - docs/operations/deploy.md
  - docs/operations/deploy-kind.md
  - docs/operations/deploy-kubernetes-minimal.md
---

# Operations Prerequisites

This page is the canonical prerequisite list for deployment and release runbooks.

## Required tooling

- `make`
- `cargo`
- `helm`
- `kubectl`

## Required access

- Cluster access for the target environment.
- Namespace and RBAC permissions required by the selected profile.
- Access to release artifacts and container images.

## Baseline checks

```bash
make ops-prereqs
```

Run this before deploy, upgrade, rollback, and release operations.
