---
title: Internal docs
audience: contributor
type: internal
stability: draft
owner: docs-governance
last_reviewed: 2026-03-01
tags:
  - governance
  - internal
related:
  - docs/_internal/governance/index.md
  - docs/_internal/meta/index.md
internal: true
---

# Internal docs

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: provide one internal sink for docs governance, generated artifacts, and contributor-only machinery.

> Contributor-only: enforcement machinery.

## Internal surfaces

- [Governance](governance/index.md)
- [Generated artifacts](generated-artifacts.md)
- [Docs metadata and operating model](meta/index.md)
- [Internal navigation freeze notes](nav/index.md)

## Usage rules

- Reader-facing pages stay outside `_internal/`.
- Generated and governance outputs are linked from contributor surfaces only.
- Internal pages are excluded from reader navigation and search-oriented entrypoints.
