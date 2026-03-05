---
title: ADR-0003 User And Dev CLI Separation
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
---

# ADR-0003: User and dev CLI separation

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Mixing product runtime commands with repository operations causes surface drift and migration confusion.

## Decision

Public runtime flows stay in user CLI; repository automation and diagnostics stay in `bijux-dev-atlas`.

## Consequences

1. Command-surface contracts guard boundaries.
2. Documentation references must map use case to the correct CLI.
