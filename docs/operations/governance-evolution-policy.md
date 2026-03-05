---
title: Governance Evolution Policy
audience: operator
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - policy
---

# Governance evolution policy

Governance enforcement rules must remain stable, explicit, and backward understandable.

## Policy

- Do not reuse rule IDs.
- New rule IDs must be appended and documented.
- Rule severity changes require rationale in the same commit.
- Rule type additions require schema updates and tests.
- Rule removals require replacement coverage in the same change.

## Change controls

- Keep `configs/governance/enforcement/rules.json` as the source of truth.
- Keep `ops/governance/enforcement/rules.snapshot.json` synchronized.
- Keep operator docs synchronized with the active rule registry.
