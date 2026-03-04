---
title: SLO Overload Survival Response
summary: Operate safely during sustained overload while protecting critical availability.
owner: Atlas Operations
stability: stable
last_reviewed: 2026-03-05
---

# SLO Overload Survival Response

## Trigger
- Alert with `runbook_id: slo-overload-survival`.

## Immediate checks
1. Confirm overload source: query surge, ingest surge, or mixed load.
2. Validate cluster safety: restart rate, disk pressure, and queue depth.
3. Check if overload is localized to one profile or region.

## Response
1. Enforce admission control and preserve core query paths.
2. Defer non-critical workloads and throttle ingest where needed.
3. Scale capacity within approved bounds and verify stabilization.

## Exit criteria
- Load and latency return below overload thresholds.
- No active saturation or restart-loop alerts remain.

## Evidence
- Attach `artifacts/ops/ops_run/observe/alerts-contract-report.json`.
- Attach `artifacts/ops/ops_run/observe/operational-readiness-report.md`.
- Attach diagnostics bundle from `bijux-dev-atlas diagnose bundle`.
