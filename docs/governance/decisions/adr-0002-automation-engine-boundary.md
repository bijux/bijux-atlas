---
title: ADR-0002 Automation Engine Boundary
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
---

# ADR-0002: Automation engine boundary

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Repository automation drifted into ad hoc script paths, reducing auditability and policy consistency.

## Decision

`bijux-dev-atlas` is the sole automation engine for repository workflows. Root `control-plane/`, `automation/`, and root automation scripts are forbidden.

## Consequences

1. All new automation is added as a dev-atlas command and tested through contracts.
2. Makefile and workflow steps stay delegation-only.
