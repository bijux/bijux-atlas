---
title: Data Classification Policy
audience: user
type: policy
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Data Classification Policy

Atlas classifies data into categories that drive storage, logging, and retention controls.

## Source of truth

- `configs/security/data-classification.yaml`

## Classification levels

- `restricted`: secrets and credentials
- `sensitive`: security and identity metadata
- `internal`: operational and diagnostics data
- `public`: externally publishable material

## Enforcement

- redaction and forbidden pattern checks for logs and artifacts
- policy validation in security command surfaces
