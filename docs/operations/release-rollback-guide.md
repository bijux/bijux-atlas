---
title: Release rollback guide
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - release
  - rollback
related:
  - docs/operations/ops-rollback-guide.md
  - docs/operations/release-upgrade-guide.md
---

# Release rollback guide

1. Identify the failed target release and the last known-good source release.
2. Run `bijux dev atlas release rollback-plan --from-version <from> --to-version <to>`.
3. Execute rollback using the plan and pinned artifacts.
4. Validate restored baseline using rollback scenario evidence outputs.
5. Record root cause, rollback evidence, and next upgrade constraints.
