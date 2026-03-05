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

## 2026-03-05: ops benchmark runner moved to dev-atlas perf domain

- Removed: `python ops/cli/perf/cli_ux_benchmark.py`
- Replacement: `bijux-dev-atlas perf cli-ux bench`
- Comparison command: `bijux-dev-atlas perf cli-ux diff <baseline> <candidate>`

Rationale: benchmark execution is governed as control-plane behavior and must not live as ad-hoc script tooling under `ops/`.
