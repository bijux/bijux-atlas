---
title: Docs Deletion Safety Checklist
audience: internal
type: policy
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - docs
  - deletion
  - safety
---

# Docs Deletion Safety Checklist

- Confirm the page is duplicated or obsolete.
- Add redirect mapping in `docs/redirects.json` when user-facing URL existed.
- Regenerate redirect maps and generated docs indexes.
- Run docs link and registry contracts.
- Update merge plan with source-to-canonical decision.
