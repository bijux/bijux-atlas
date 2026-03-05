---
title: Migration Completion
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - migration
---

# Migration completion

The repository migration to dev-atlas-owned automation is complete when:

1. `bijux-dev-atlas migrations status --format json` returns `status=ok`.
2. `bijux-dev-atlas checks automation-boundaries` returns `status=pass`.
3. `bijux-dev-atlas contract automation-boundaries` returns `status=pass`.

The status report artifact is written to:

- `artifacts/governance/migration-complete-status.json`
