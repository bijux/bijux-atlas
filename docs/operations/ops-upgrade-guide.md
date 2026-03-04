---
title: Ops upgrade guide
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - upgrade
related:
  - docs/operations/upgrade-procedure.md
---

# Ops upgrade guide

1. Read release notes and compatibility policy.
2. Pull OCI chart for the target version.
3. Choose the profile values file from the release bundle.
4. Run `helm upgrade` with explicit chart version and values.
5. Verify `/readyz`, `/healthz`, and metrics.
6. Archive upgrade evidence with release artifacts.
