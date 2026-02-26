# Operations Observability Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `docs-governance`

## What

Index page for `operations/observability` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

- [Compatibility Promise](compatibility.md)
- [SLO](slo.md)
- [Tracing](tracing.md)
- [Metric Cardinality Guardrails](metric-cardinality-guardrails.md)
- [SLO-Critical Metrics](slo-critical-metrics.md)
- [Dashboard](dashboard.md)
- [Alerts](alerts.md)
- [Runbook Map](runbook-dashboard-alert-map.md)
- [Acceptance Gates](acceptance-gates.md)
- [Drills](drills.md)
- [Telemetry Compatibility](telemetry-compatibility.md)
- [Observability Acceptance Checklist](acceptance-checklist.md)
- [Debug P99 Latency](debug-p99-latency.md)
- [Debug Overload](debug-overload.md)
- [Observability Profiles](profiles.md)
- [Trace Sampling Policy](trace-sampling-policy.md)
- [OTEL Outage Behavior](otel-outage-behavior.md)
- [Metric Ownership](metric-ownership.md)
- [Generated Surface](../../_generated/observability-surface.md)

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-ci`
