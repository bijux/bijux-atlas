---
title: Documentation Guide
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - onboarding
related:
  - docs/INDEX.md
  - docs/start-here.md
---

# Documentation Guide

Canonical navigation authority: `docs/INDEX.md`.

Docs layers:

- Narrative guides under `docs/{product,architecture,operations,tutorials}`.
- Reference pages under `docs/reference`.
- Generated surfaces under `docs/_generated` and `docs/_internal/generated`.

Where to start:

- New readers: [Start Here](start-here.md)
- Platform overview: [Architecture Index](architecture/index.md)
- Runtime operations: [Operations Index](operations/index.md)
- CLI/API reference: [Reference Index](reference/index.md)

Generated content model:

- `docs/_generated` is generator-owned snippet content used by tutorials.
- `docs/_internal/generated` is ops/governance/repository/report output and internal evidence.
- Manual edits to generated markdown are forbidden.
