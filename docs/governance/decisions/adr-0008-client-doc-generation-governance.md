---
title: ADR-0008 Client Documentation Generation Governance
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
  - clients
---

# ADR-0008: Client documentation generation governance

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Client SDK docs and verification logic drifted into client-local tooling, creating duplicate automation surfaces and inconsistent governance behavior.

## Decision

Client documentation generation and verification are responsibilities of `bijux-dev-atlas`. Client-local scripts are replaced by governed `bijux-dev-atlas clients ...` commands.

## Consequences

1. Client docs and examples are generated and verified through a single control-plane.
2. Python remains allowed for SDK product code, not for repository automation orchestration.

