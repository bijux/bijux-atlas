---
title: SLO Standard Burn Response
summary: Respond to medium-severity SLO burn-rate alerts requiring active mitigation.
owner: Atlas Operations
stability: stable
last_reviewed: 2026-03-05
---

# SLO Standard Burn Response

## Trigger
- Alerts with `runbook_id: slo-standard-burn`.

## Immediate checks
1. Identify affected SLO and service scope from alert labels.
2. Confirm whether symptoms are latency, errors, or availability loss.
3. Check for correlated infrastructure signals (disk, restart loops, ingest failures).

## Response
1. Initiate incident channel and assign incident lead.
2. Apply mitigations: reduce ingest pressure, shed non-critical traffic, or rollback risky config.
3. Track SLO burn slope every 10 minutes and update incident log.

## Exit criteria
- Burn-rate alert remains resolved for 60 minutes.
- No active high-severity dependent alerts remain.

## Evidence
- Attach `artifacts/ops/ops_run/observe/operational-readiness-report.json`.
- Attach `artifacts/ops/ops_run/observe/slo-contract-report.json`.
- Attach `artifacts/ops/ops_run/observe/runbooks-contract-report.json`.
