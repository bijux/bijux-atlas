---
title: Docs Surface Boundaries
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - governance
  - boundaries
related:
  - docs/governance/docs-ssot-rule.md
  - docs/governance/docs-minimization-policy.md
---

# Docs Surface Boundaries

This page defines where narrative documentation is allowed.

## Public docs root

- Canonical public docs live under `docs/**`.
- Public navigation starts from `docs/INDEX.md`.
- Operational narratives live under `docs/operations/**`.

## Internal docs surfaces

- Internal governance docs live under `docs/_internal/**`.
- Generated docs evidence lives under `docs/_internal/generated/**`.
- Tutorial snippet outputs live under `docs/_generated/**`.

## Repository non-doc roots

- `ops/**` contains operational artifacts, contracts, inventories, and runbook templates.
- `configs/**` contains configuration artifacts and schema inputs.
- `crates/bijux-dev-atlas/docs/**` contains crate-local control-plane documentation.

Rule: non-doc roots may provide short boundary READMEs, but durable narratives belong under `docs/**`.
