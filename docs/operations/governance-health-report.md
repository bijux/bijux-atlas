---
title: Governance Health Report
audience: operator
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - report
---

# Governance health report

Generate an aggregated governance status report:

```bash
bijux-dev-atlas governance report --format json
```

The command writes `artifacts/governance/governance-health-report.json` and returns a summary payload with status, enforcement results, contributor guideline validation, governance documentation validation, and ADR index coverage.
