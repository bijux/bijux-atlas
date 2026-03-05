---
title: ADR-0001 Governance Authority Boundaries
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
---

# ADR-0001: Governance authority boundaries

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance
- Related contracts: GOV-RULE-001, GOV-RULE-003

## Context

Governance ownership and decision rights must be explicit for long-term maintainability.

## Decision

Define domain ownership with named primary and backup maintainers and require cross-domain signoff for cross-domain changes.

## Alternatives considered

- Ad hoc ownership per pull request
- Single global owner for all domains

## Consequences

- Clear review and approval paths
- Better continuity during maintainer rotation

## Rollout and validation

- Publish governance model and maintainer registry
- Validate ownership references through governance checks
