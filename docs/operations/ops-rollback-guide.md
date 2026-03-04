---
title: Ops rollback guide
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - rollback
related:
  - docs/operations/rollback-procedure.md
---

# Ops rollback guide

1. Identify last known-good release chart and values profile.
2. Run `helm rollback` or reinstall using pinned chart digest/tag.
3. Validate service health and data path checks.
4. Capture rollback evidence and incident context.
5. Open follow-up change record before retrying upgrade.
