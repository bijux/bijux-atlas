---
title: Tutorial: Deploy Runtime with Helm
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - helm
  - deploy
related:
  - docs/operations/deploy-kubernetes-minimal.md
  - docs/operations/observability/dashboard-installation-guide.md
---

# Tutorial: Deploy Runtime with Helm

## Goal

Deploy Atlas runtime into Kubernetes using Helm values.

## Steps

1. Select a base values file and adjust image/config refs.
2. Run Helm install or upgrade for target namespace.
3. Validate pod readiness and service reachability.

## Expected result

Cluster runtime is reachable and ready for ingest/query workloads.
