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
- Metrics contract must pass: `crates/bijux-dev-atlas/src/observability/contracts/metrics/check_metrics_contract.py`.
- Dashboard contract must pass: `crates/bijux-dev-atlas/src/observability/contracts/dashboards/check_dashboard_contract.py`.
- Alerts contract must pass: `crates/bijux-dev-atlas/src/observability/contracts/alerts/check_alerts_contract.py`.
- Tracing contract is optional unless OTEL enabled: `crates/bijux-dev-atlas/src/observability/contracts/tracing/check_tracing_contract.py`.
- Runtime cardinality guardrail must pass: `ops-metrics-check`.
- Logs schema must pass: `ops-metrics-check`.
- K8s log gate must validate schema: `ops/k8s/tests/checksuite/checks/obs/runtime/test_logs_json.sh`.
- Drill scripts must assert signal transitions:
  - alerts: `ops-drill-alerts`
  - overload: `ops-drill-overload`
  - store outage: `ops-drill-store-outage`
  - memory growth: `ops-drill-memory-growth`

## Failure modes

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

- Profile-specific install target: `ops-obs-mode` (`ATLAS_OBS_PROFILE=local-compose|kind|cluster`).
- Compatibility profile aliases: `ops-obs-mode-minimal`, `ops-obs-mode-full`.
- Logs schema contract: `ops/observe/contract/logs-fields-contract.json`.
- Failure artifacts on validation error: `artifacts/ops/observe/validate-fail-<timestamp>/`.
