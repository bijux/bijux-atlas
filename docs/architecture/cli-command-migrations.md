---
title: CLI Command Migration Notes
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - cli
  - migration
---

# CLI command migration notes

## 2026-03-05: runtime diagnostics moved to developer CLI

- `self-check` moved from `bijux-atlas` to `bijux-dev-atlas runtime self-check`.
- `print-config-schema` moved from `bijux-atlas` to `bijux-dev-atlas runtime print-config-schema`.
- New explanatory command: `bijux-dev-atlas runtime explain-config-schema`.

Rationale: runtime diagnostics are repository operations, not end-user product flows.
