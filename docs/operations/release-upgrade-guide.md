---
title: Release upgrade guide
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - release
  - upgrade
related:
  - docs/operations/ops-upgrade-guide.md
  - docs/operations/upgrade-compatibility-guide.md
---

# Release upgrade guide

1. Confirm source and target versions are listed in `ops/e2e/scenarios/upgrade/version-compatibility.json`.
2. Run `bijux dev atlas release compatibility-check --from-version <from> --to-version <to>`.
3. Run `bijux dev atlas release upgrade-plan --from-version <from> --to-version <to>` and follow the generated steps.
4. Execute the upgrade scenario for the same transition in evidence mode.
5. Validate health, API responses, and dataset registry after upgrade.
6. Archive upgrade evidence artifacts with the release record.
