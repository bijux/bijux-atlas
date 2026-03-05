---
title: Documentation Architecture
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - architecture
related:
  - docs/INDEX.md
  - docs/governance/docs-ssot-rule.md
---

# Documentation Architecture

Documentation is split into three layers:

## Narrative

Reader-oriented guides and explanations.

- product intent: `docs/product/**`
- system design: `docs/architecture/**`
- operations: `docs/operations/**`
- tutorials: `docs/tutorials/**`

## Reference

Lookup-oriented surfaces with stable terms and command/API details.

- `docs/reference/**`
- `docs/api/**`
- `docs/cli/**`

## Generated

Machine-produced content used for evidence and snippets.

- tutorial snippet outputs: `docs/_generated/**`
- governance evidence outputs: `docs/_internal/generated/**`

Design rule: narrative and reference pages link to generated outputs; they do not duplicate generated tables or command dumps.
