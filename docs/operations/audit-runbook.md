---
title: Audit Runbook
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - audit
  - runbook
---

# Audit runbook

1. Run `bijux-dev-atlas audit run --format json`.
2. Review `failure_classification` and `metrics` in `artifacts/audit/run-report.json`.
3. Run `bijux-dev-atlas audit report --format json` for a packaged report.
4. Run `bijux-dev-atlas audit explain --format json` when schema/check semantics are needed.
5. Record the run in evidence bundle collection using `ops/audit/evidence-integration.json`.
