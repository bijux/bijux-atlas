---
title: Governance Enforcement Workflow
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - workflow
---

# Governance enforcement workflow

## Local workflow

1. Run `bijux-dev-atlas governance check --format json`.
2. Resolve all violations.
3. Run `bijux-dev-atlas governance validate --format json`.
4. Confirm updated governance artifacts under `artifacts/governance/`.

## Change workflow

1. Update `configs/governance/enforcement/rules.json`.
2. Update `configs/governance/enforcement/rules.schema.json` if rule types changed.
3. Refresh `ops/governance/enforcement/rules.snapshot.json`.
4. Update `docs/operations/governance-rule-reference.md`.
5. Run targeted governance tests.
