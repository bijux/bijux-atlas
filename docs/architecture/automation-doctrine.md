---
title: Automation Doctrine
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - automation
  - governance
---

# Automation doctrine

`bijux-dev-atlas` is the only repository automation engine.

## Rules

- Do not add root `tools/` or `scripts/` automation trees.
- Do not add root `*.sh` or `*.py` automation helpers.
- Make targets must delegate to `bijux-dev-atlas` command surfaces.
- Workflows must call `bijux-dev-atlas` commands for repository automation.

## Rationale

Single ownership of automation keeps contracts, audits, and governance controls deterministic.
