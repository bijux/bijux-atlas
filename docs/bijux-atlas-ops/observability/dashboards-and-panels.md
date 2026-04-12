---
title: Dashboards and Panels
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Dashboards and Panels

Grafana dashboards under `ops/observe/dashboards/` capture the curated
operational views for Atlas runtime and supporting systems.

## Purpose

Use this page to understand which dashboards are canonical, how dashboard
acceptance is validated, and what panel classes operators should expect to find.

## Source of Truth

- `ops/observe/dashboard-registry.json`
- `ops/observe/dashboards/`
- `ops/observe/contracts/dashboard-json-validation-contract.json`
- `ops/observe/contracts/dashboard-panels-contract.json`
- `ops/observe/dashboard-metadata.schema.json`

## Dashboard Registry

`ops/observe/dashboard-registry.json` currently defines canonical dashboards for
runtime health, query performance, ingest pipeline, artifact registry, system
resources, SLO compliance, error classification, latency distribution, artifact
cache performance, and drift detection.

## Canonical Versus Fixture Dashboards

- canonical dashboards live in the main dashboard files such as
  `atlas-runtime-health-dashboard.json` and
  `atlas-query-performance-dashboard.json`
- fixture dashboards such as `fixtures/minimal-dashboard.json` exist to validate
  contract handling and should not be treated as production operator views
- `atlas-observability-dashboard.golden.json` acts as a golden validation target
  rather than a casual edit surface

## Required Panel Classes

Operators should expect the accepted dashboards to cover panel classes for:

- runtime health and readiness
- latency, error, and throughput
- resource saturation
- store, registry, and cache behavior
- SLO or drift summary views where relevant
