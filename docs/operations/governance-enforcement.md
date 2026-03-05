---
title: Governance Enforcement
audience: operator
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - operations
  - enforcement
---

# Governance enforcement

`bijux-dev-atlas governance check` evaluates repository governance rules from `configs/governance/enforcement/rules.json`.

## Command

```bash
bijux-dev-atlas governance check --format json
```

## Outputs

- `artifacts/governance/enforcement-report.json`
- `artifacts/governance/enforcement-coverage.json`

## Enforcement scope

- Required file presence
- Prohibited file absence
- Repository layout conformance
- Documentation front matter completeness
- Registry completeness for contracts, checks, scenarios, ops artifacts, and release artifacts
- Documentation navigation consistency
