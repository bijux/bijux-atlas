---
title: Tutorial: Debug Pipeline
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - tutorial
  - debugging
related:
  - docs/operations/incident-response.md
  - docs/operations/runbooks/index.md
---

# Tutorial: Debug Pipeline

## Goal

Diagnose failures in ingest, validation, or promotion pipeline stages.

## Steps

1. Identify failed stage from logs and check reports.
2. Correlate failure with contract/check identifiers.
3. Apply corrective change in owning layer.
4. Re-run affected checks and full workflow validation.

## Expected result

Root cause is documented, remediation is applied, and workflow returns to passing state.
