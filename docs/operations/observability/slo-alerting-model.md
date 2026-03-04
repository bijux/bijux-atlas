---
title: SLOs and alerting model
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# SLOs and alerting model

This page defines the operational SLO baseline and how alerts map to those objectives.

## SLO set

- `atlas.availability`: objective `99.9%` over `30d`
- `atlas.p95_latency`: objective `0.30s` over `30d`
- `atlas.error_rate`: objective `0.5%` over `30d`
- `atlas.ingest_throughput`: objective `50 records/s` over `10m`

Source files:
- `ops/observe/slo-definitions.json`
- `ops/observe/slo-measurement.json`
- `ops/observe/slo-metric-map.json`

## Alert model

Alert rules are defined in:
- `ops/observe/alerts/atlas-alert-rules.yaml`
- `ops/observe/alerts/slo-burn-rules.yaml`

Each alert must include:
- `severity`
- `subsystem`
- `alert_contract_version`
- `annotations.runbook`

## Verification

- `bijux dev atlas ops observe verify`
- `bijux dev atlas ops observe alerts verify`
- `bijux dev atlas ops observe runbooks verify`
