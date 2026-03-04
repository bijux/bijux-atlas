---
title: SLO Cheap Burn Response
summary: Respond to low-severity but sustained SLO burn-rate alerts.
owner: Atlas Operations
stability: stable
last_reviewed: 2026-03-05
---

# SLO Cheap Burn Response

## Trigger
- Alerts with `runbook_id: slo-cheap-burn`.

## Immediate checks
1. Confirm which SLO objective is burning from the alert labels.
2. Inspect the last 60 minutes of request volume and error/latency signals.
3. Verify whether the burn is isolated to one route, dataset, or release.

## Response
1. If a recent deploy exists, pause rollout and compare with previous release behavior.
2. If burn is traffic-driven, apply safe rate controls and protect critical endpoints.
3. If burn is data-path driven, reduce ingest concurrency until error budget drain stabilizes.

## Exit criteria
- Burn-rate alert clears for at least 30 minutes.
- Error budget trend returns inside expected operating envelope.

## Evidence
- Attach `artifacts/ops/ops_run/observe/operational-readiness-report.json`.
- Attach `artifacts/ops/ops_run/observe/alerts-contract-report.json`.
- Attach incident diagnostics bundle from `bijux-dev-atlas diagnose bundle`.
