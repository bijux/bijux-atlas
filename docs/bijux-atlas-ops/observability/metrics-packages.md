---
title: Metrics Packages
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Metrics Packages

Metrics-related sources span runtime metrics, Prometheus scrape expectations,
SLO definitions, and dashboard panels.

## Purpose

Use this page to understand the Atlas metrics families, their ownership
boundaries, and the review requirements for adding or changing a metric.

## Source of Truth

- `ops/observe/metrics/registry.snapshot.json`
- `ops/observe/metrics/label-cardinality-budget.json`
- `ops/observe/contracts/metrics-contract.json`
- `ops/observe/contracts/metrics.golden.prom`
- `ops/observe/slo-definitions.json`
- `ops/observe/slo-metric-map.json`

## Package Boundaries

Metrics in Atlas are governed in packages rather than as isolated counters:

- runtime and request health metrics
- latency and error-budget metrics
- ingest and store-related metrics
- readiness and overload metrics
- SLO measurement metrics used by alerts and dashboards

## Cardinality Budget Policy

`ops/observe/metrics/label-cardinality-budget.json` currently caps the label
budget at `200` and allowlists labels such as `dataset`, `route`,
`status_code`, `subsystem`, `operation`, and `version`. New metrics that widen
label space need explicit review before they are accepted.

## Review Requirements for New Metrics

When adding a metric, review:

- whether the registry snapshot needs to change
- whether the metric fits an existing package or creates a new operator surface
- whether the label set stays inside the cardinality budget
- whether dashboards, alerts, or SLO rules need updating

## Related Contracts and Assets

- `ops/observe/metrics/`
- `ops/observe/slo-definitions.json`
- `ops/observe/slo-metric-map.json`
- `ops/observe/metrics/registry.snapshot.json`
- `ops/observe/metrics/label-cardinality-budget.json`
