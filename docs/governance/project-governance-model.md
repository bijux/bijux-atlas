---
title: Project Governance Model
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - ownership
---

# Project governance model

Atlas is governed by explicit ownership domains with accountable maintainers, published policies, and verifiable controls.

## Governance domains

- Architecture and contracts
- Security and compliance
- Runtime and operations
- Documentation and developer experience

## Ownership model

- Each domain has a designated owner and backup maintainer.
- Governance changes require reviewer approval from the owning domain.
- Cross-domain changes require at least one reviewer from each affected domain.

## Control model

- Policy sources live under `docs/governance/` and `configs/governance/`.
- Rule enforcement is executed by `bijux-dev-atlas governance check`.
- Governance integrity is evaluated by `bijux-dev-atlas governance validate`.
