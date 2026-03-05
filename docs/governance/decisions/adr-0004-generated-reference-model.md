---
title: ADR-0004 Generated Reference Model
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
---

# ADR-0004: Generated reference model

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Manual pasted command output in docs causes drift and duplicate maintenance.

## Decision

Reference-heavy tutorial output is generated and validated through governed dev-atlas generation and verification commands.

## Consequences

1. Generated snippets become deterministic contract artifacts.
2. Manual edits to generated blocks are rejected by checks.
