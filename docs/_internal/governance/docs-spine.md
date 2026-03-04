---
title: Docs Spine
audience: internal
type: policy
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - docs
  - navigation
  - governance
---

# Docs Spine

Canonical reader spine pages:

- `docs/start-here.md`
- `docs/product/index.md`
- `docs/operations/index.md`
- `docs/development/index.md`
- `docs/control-plane/index.md`
- `docs/reference/index.md`

Supplemental canonical entrypoints:

- `docs/index.md`
- `docs/architecture/index.md`
- `docs/api/index.md`

## Policy

- Every published page outside the spine must link upward to one spine entrypoint.
- Top-level navigation categories are capped at `13`.
- New top-level navigation categories require docs-governance approval.
- Internal pages must stay under `docs/_internal/`.
