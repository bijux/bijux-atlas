---
title: Doctrine Compliance Report
audience: contributor
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - automation
---

# Doctrine compliance report

Generate a consolidated automation doctrine compliance report:

```bash
bijux-dev-atlas governance doctrine-report --format json
```

Output artifact:

- `artifacts/governance/doctrine-compliance-report.json`

The report consolidates:

1. `checks automation-boundaries`
2. `contract automation-boundaries`
3. `migrations status`

It exists to provide a single reviewer-facing signal that the repository still honors delegation-first automation boundaries.
