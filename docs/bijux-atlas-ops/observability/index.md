---
title: Observability
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Observability

`bijux-atlas-ops/observability` explains how Atlas turns logs, metrics, traces,
dashboards, alerts, drills, and evidence outputs into an operable signal pack.

## Purpose

Use this section to understand how runtime instrumentation connects to alerting,
readiness, SLO measurement, incident response, and release or rollout evidence.

## Source of Truth

- `ops/observe/alerts/` and `ops/observe/alert-catalog.json`
- `ops/observe/dashboards/` and `ops/observe/dashboard-registry.json`
- `ops/observe/contracts/`
- `ops/observe/metrics/`
- `ops/observe/tracing/`
- `ops/observe/drills/`, `ops/observe/drills.json`, and
  `ops/observe/telemetry-drills.json`
- `ops/observe/generated/telemetry-index.json`

## Observability Operating Model

Atlas observability has three primary signals:

- logs capture structured events and error context
- metrics capture aggregate behavior, SLO measurements, and alert triggers
- traces capture request-level path and correlation

Those signals are packaged with dashboards, alert rules, drills, and generated
indexes so operators can validate not only that telemetry exists, but that it is
usable during rollout and failure.

## Evidence Produced

This section points operators to:

- alert catalogs and rule packs
- dashboard registry and validation outputs
- readiness and SLO measurement artifacts
- telemetry drill definitions and schema-backed results
- generated telemetry indexes used in change review

## Pages

- [Alert Rules](alert-rules.md)
- [Dashboards and Panels](dashboards-and-panels.md)
- [Health, Readiness, and Drain](health-readiness-and-drain.md)
- [Incident Response](incident-response.md)
- [Logging Contracts](logging-contracts.md)
- [Logging, Metrics, and Tracing](logging-metrics-and-tracing.md)
- [Metrics Packages](metrics-packages.md)
- [Operational Evidence Reports](operational-evidence-reports.md)
- [Telemetry Drills](telemetry-drills.md)
- [Tracing Pipelines](tracing-pipelines.md)
