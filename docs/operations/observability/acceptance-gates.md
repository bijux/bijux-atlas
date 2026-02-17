# Observability Acceptance Gates

- Owner: `bijux-atlas-operations`

## What

Defines the minimum observability checks required before production rollout.

## Why

Prevents deployments with missing telemetry coverage or invalid alerting assets.

## Contracts

- Metrics contract must pass: `scripts/observability/check_metrics_contract.py`.
- Dashboard contract must pass: `scripts/observability/check_dashboard_contract.py`.
- Alerts contract must pass: `scripts/observability/check_alerts_contract.py`.
- Tracing contract must pass: `scripts/observability/check_tracing_contract.py`.
- Runtime cardinality guardrail must pass: `ops/observability/scripts/check_metric_cardinality.py`.
- Logs schema must pass: `ops/observability/scripts/validate_logs_schema.py`.

## Failure Modes

Missing metrics, stale dashboards, invalid alert expressions, or schema-drifted logs.

## How to verify

```bash
$ make ops-observability-validate
$ make ops-observability-smoke
```

Expected output: both targets exit 0 and produce contract checks without warnings.

## See also

- [Observability Index](INDEX.md)
- [Alerts](alerts.md)
- [Dashboard](dashboard.md)
- [Tracing](tracing.md)
