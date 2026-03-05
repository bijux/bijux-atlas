---
title: Docs SSOT Rule
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - ssot
  - governance
related:
  - docs/INDEX.md
  - docs/README.md
  - docs/architecture/docs-architecture.md
---

# Docs SSOT Rule

Documentation single-source-of-truth rules:

- `docs/INDEX.md` is the canonical top-level navigation authority.
- Reader guidance is authored in narrative/reference pages under `docs/**` (excluding internal and generated sinks).
- Internal governance sources live under `docs/_internal/**`.
- Generated references live under `docs/_generated/**` and `docs/_internal/generated/**`.
- Generated pages are derived artifacts; they are not manually authored narrative SSOT.

Authority order:

1. Governance policy pages in `docs/_internal/governance/**`
2. Canonical reader pages linked from `docs/INDEX.md`
3. Generated evidence and snippet outputs
