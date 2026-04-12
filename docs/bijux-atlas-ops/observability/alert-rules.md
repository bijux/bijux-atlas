---
title: Alert Rules
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Alert Rules

Alerting rules and coverage gates live under `ops/observe/rules/` and
`ops/observe/alerts/` so noise and signal can be reviewed together.

## Purpose

Use this page to understand which alert packs Atlas maintains, how alerts are
classified, and what an operator is expected to do when one fires.

## Source of Truth

- `ops/observe/alerts/atlas-alert-rules.yaml`
- `ops/observe/alerts/security-alert-rules.yaml`
- `ops/observe/alerts/slo-burn-rules.yaml`
- `ops/observe/alert-catalog.json`
- `ops/observe/contracts/alerts-contract.json`

## Alert Inventory

`ops/observe/alert-catalog.json` currently records alert identities such as:

- critical service alerts like `api.error-rate-high`,
  `BijuxAtlasHigh5xxRate`, `BijuxAtlasStoreDownloadFailures`,
  `BijuxAtlasOverloadSurvivalViolated`, and
  `BijuxAtlasStoreBackendErrorSpike`
- warning alerts like `api.latency-p95-high`,
  `BijuxAtlasP95LatencyRegression`, `BijuxAtlasCacheThrash`,
  `BijuxAtlasRegistryRefreshStale`, and the fast, medium, and slow SLO burn
  alerts

## Alert Classes

Use these operator classes when reviewing the rule packs:

- runtime availability and latency alerts
- store and registry dependency alerts
- overload or degradation alerts
- SLO burn alerts
- security alerts

## Expected Operator Action

- critical alerts require immediate triage, dashboard review, and incident
  evidence capture
- warning alerts require trend review, contract verification, and a decision on
  whether the issue is becoming release-blocking
- SLO burn alerts should trigger service-behavior review even if the system is
  still technically up
