---
title: SLO Registry Refresh Stale Response
summary: Resolve stale registry refresh conditions before they impact serving guarantees.
owner: Atlas Operations
stability: stable
last_reviewed: 2026-03-05
---

# SLO Registry Refresh Stale Response

## Trigger
- Alert with `runbook_id: slo-registry-refresh-stale`.

## Immediate checks
1. Confirm last successful registry refresh timestamp.
2. Validate artifact source reachability and integrity checks.
3. Inspect recent release or config changes affecting refresh cadence.

## Response
1. Retry refresh with diagnostic logging enabled.
2. If source-side outage exists, switch to approved fallback artifact mirror.
3. If integrity mismatches occur, block adoption and escalate incident.

## Exit criteria
- Registry refresh timestamp is current and advancing.
- Related stale-refresh alerts clear and stay resolved.

## Evidence
- Attach `artifacts/ops/ops_run/observe/alerts-contract-report.json`.
- Attach `artifacts/ops/ops_run/observe/operational-readiness-report.json`.
- Attach integrity or refresh diagnostics from the incident bundle.
