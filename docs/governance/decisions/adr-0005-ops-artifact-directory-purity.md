---
title: ADR-0005 Ops Artifact Directory Purity
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
---

# ADR-0005: Ops artifact directory purity

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Narrative documentation mixed into artifact directories increases boundary ambiguity.

## Decision

`ops/` and `configs/` contain operational artifacts only. Narrative documentation lives under `docs/`.

## Consequences

1. Boundary checks can be strict and deterministic.
2. Operational directories stay tool-friendly and low-noise.
