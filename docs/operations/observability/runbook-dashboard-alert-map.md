# Runbook Dashboard Alert Map

- Owner: `bijux-atlas-operations`

## What

Maps incident runbooks to the primary dashboard panels and alert rules used during triage.

## Why

Reduces incident response latency by making first-hop telemetry explicit.

## Contracts

- Dashboard source: `ops/observability/grafana/atlas-observability-dashboard.json`
- Alert source: `ops/observability/alerts/atlas-alert-rules.yaml`
- Validation target: `ops-observability-validate`.
- Runbook sources: `docs/operations/runbooks/*.md`

| Runbook | Dashboard Panels | Alerts |
|---|---|---|
| `store-outage.md` | `Store Open/Download p95`, `Dataset Cache Hit/Miss` | `BijuxAtlasStoreDownloadFailures`, `BijuxAtlasCacheThrash` |
| `high-memory.md` | `Dataset Cache Size`, `HTTP p95 Latency by Route` | `BijuxAtlasP95LatencyRegression` |
| `incident-playbook.md` | `HTTP Request Rate by Route/Status`, `SLO Burn Rate (5xx, 5m/1h)` | `BijuxAtlasHigh5xxRate` |
| `load-failure-triage.md` | `HTTP p95 Latency by Route`, `SQLite Query p95 by Class` | `BijuxAtlasP95LatencyRegression`, `BijuxAtlasP99LatencyRegression` |

## Failure modes

Missing mapping causes slow triage and inconsistent alert handling.

## How to verify

```bash
make ops-dashboards-validate
make ops-alerts-validate
make observability-check
make ops-observability-validate
```

Expected output: contracts and runbook links pass checks.

## See also

- [Observability Index](INDEX.md)
- [Alerts](alerts.md)
- [Dashboard](dashboard.md)
- [Acceptance Gates](acceptance-gates.md)
