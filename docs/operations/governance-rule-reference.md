---
title: Governance Rule Reference
audience: operator
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - rules
---

# Governance rule reference

Canonical registry: `configs/governance/enforcement/rules.json`

## Rule IDs

- `GOV-RULE-001`: required files must exist
- `GOV-RULE-002`: prohibited files must not exist
- `GOV-RULE-003`: repo layout must match contract
- `GOV-RULE-004`: docs front matter completeness
- `GOV-RULE-005`: contract registry completeness
- `GOV-RULE-006`: checks registry completeness
- `GOV-RULE-007`: scenario registry completeness
- `GOV-RULE-008`: ops artifact registry completeness
- `GOV-RULE-009`: release artifact registry completeness
- `GOV-RULE-010`: docs navigation consistency

Use `bijux-dev-atlas governance rules --format json` for machine-readable rule metadata.
