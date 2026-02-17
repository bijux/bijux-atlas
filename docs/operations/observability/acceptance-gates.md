# Observability Acceptance Gates

- Owner: `bijux-atlas-operations`

## What

Defines the minimum observability checks required before production rollout.

## Why

Prevents deployments with missing telemetry coverage or invalid alerting assets.

## Contracts

- Primary gate target: `ops-observability-validate`.
- Smoke gate target: `ops-observability-smoke`.
- Install/uninstall targets: `ops-obs-install`, `ops-obs-uninstall`.
- Alias validate target: `ops-obs-validate`.
- Metrics contract must pass: `scripts/observability/check_metrics_contract.py`.
- Dashboard contract must pass: `scripts/observability/check_dashboard_contract.py`.
- Alerts contract must pass: `scripts/observability/check_alerts_contract.py`.
- Tracing contract is optional unless OTEL enabled: `scripts/observability/check_tracing_contract.py`.
- Runtime cardinality guardrail must pass: `ops/observability/scripts/check_metric_cardinality.py`.
- Logs schema must pass: `ops/observability/scripts/validate_logs_schema.py`.
- K8s log gate must validate schema: `ops/e2e/k8s/tests/test_logs_json.sh`.
- Drill scripts must assert signal transitions:
  - alerts: `ops/observability/scripts/drill_alerts.sh`
  - overload: `ops/observability/scripts/drill_overload.sh`
  - store outage: `ops/observability/scripts/drill_store_outage.sh`
  - memory growth: `ops/observability/scripts/drill_memory_growth.sh`

## Failure Modes

Missing metrics, stale dashboards, invalid alert expressions, or schema-drifted logs.

## How to verify

```bash
$ make ops-observability-validate
$ make ops-observability-smoke
$ make ops-obs-install
$ make ops-obs-uninstall
```

Expected output: both targets exit 0 and produce contract checks without warnings.

## See also

- [Observability Index](INDEX.md)
- [Alerts](alerts.md)
- [Dashboard](dashboard.md)
- [Tracing](tracing.md)

- Mode-specific install target: `ops-obs-mode` (`ATLAS_OBS_MODE=minimal|full`).
- Compatibility mode targets: `ops-obs-mode-minimal`, `ops-obs-mode-full`.
- Logs schema contract: `ops/observability/contract/logs-fields-contract.json`.
- Failure artifacts on validation error: `artifacts/ops/observability/validate-fail-<timestamp>/`.
