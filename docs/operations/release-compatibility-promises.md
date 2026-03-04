---
title: Release compatibility promises
audience: operators
type: concept
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - release
  - compatibility
related:
  - docs/operations/ops-compatibility-promise.md
  - docs/operations/upgrade-compatibility-guide.md
---

# Release compatibility promises

- Patch upgrades must preserve API and schema compatibility.
- Minor upgrades may add capabilities but must provide migration paths.
- Incompatible schema or API transitions require a major version boundary.
- Every release transition must be represented in the upgrade compatibility table.
- Rollback scenarios must prove restoration to baseline behavior.
