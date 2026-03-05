---
title: Docs Minimization Policy
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - governance
  - policy
related:
  - docs/architecture/docs-architecture.md
  - docs/tutorials/consolidation-roadmap.md
---

# Docs Minimization Policy

Minimization rules:

- Prefer extending a canonical page over creating a new sibling page.
- When two pages answer the same question, merge into one canonical page and keep redirects.
- Keep command inventories and large tables in generated pages.
- Keep narrative pages short and link to references for detail.
- Delete dead pages instead of preserving stale placeholders.

Merge decision guide:

- Same audience + same intent => merge.
- Same topic + different format (guide vs reference) => split only when both add clear value.
- Repeated command output => move to generated snippets.

Governance checks:

- docs page metadata (`type`, `owner`) is required for reader pages.
- dead page and duplication reports are part of docs governance checks.
- `docs prune-plan`, `docs dedupe-report`, and `docs toc verify` drive consolidation operations.
