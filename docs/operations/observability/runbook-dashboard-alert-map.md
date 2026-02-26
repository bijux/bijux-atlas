# Runbook Dashboard Alert Map

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Maps incident runbooks to the primary dashboard panels and alert rules used during triage.

## Why

Reduces incident response latency by making first-hop telemetry explicit.

## Contracts

- Dashboard source: `ops/observe/grafana/atlas-observability-dashboard.json`
- Alert source: `ops/observe/alerts/atlas-alert-rules.yaml`
- Validation target: `ops-observability-validate`.
- Runbook sources: `docs/operations/runbooks/*.md`

| Runbook | Dashboard Panels | Alerts |
|---|---|---|
| `store-outage.md` | `Store Open/Download p95`, `Dataset Cache Hit/Miss` | `BijuxAtlasStoreDownloadFailures`, `BijuxAtlasCacheThrash` |
| `dataset-corruption.md` | `Dataset Cache Hit/Miss`, `Store Open/Download p95` | `BijuxAtlasStoreDownloadFailures`, `BijuxAtlasCorruptedArtifact` |
| `high-memory.md` | `Dataset Cache Size`, `HTTP p95 Latency by Route` | `BijuxAtlasP95LatencyRegression` |
| `incident-playbook.md` | `HTTP Request Rate by Route/Status`, `SLO Burn Rate (5xx, 5m/1h)` | `BijuxAtlasHigh5xxRate` |
| `k8s-perf-chaos.md` | `HTTP p95 Latency by Route`, `CPU Saturation`, `Memory RSS` | `BijuxAtlasP99LatencyRegression`, `BijuxAtlasHigh5xxRate` |
| `load-failure-triage.md` | `HTTP p95 Latency by Route`, `SQLite Query p95 by Class` | `BijuxAtlasP95LatencyRegression`, `BijuxAtlasP99LatencyRegression` |
| `memory-profile-under-load.md` | `Dataset Cache Size`, `Memory RSS`, `GC Pause` | `BijuxAtlasP95LatencyRegression` |
| `pod-churn.md` | `Pod Restart Count`, `HTTP Request Rate by Route/Status` | `BijuxAtlasHigh5xxRate`, `BijuxAtlasP95LatencyRegression` |
| `profile-under-load.md` | `HTTP p95 Latency by Route`, `SQLite Query p95 by Class` | `BijuxAtlasP99LatencyRegression` |
| `registry-federation.md` | `Catalog Refresh Duration`, `Store Open/Download p95` | `BijuxAtlasStoreDownloadFailures` |
| `rollback-playbook.md` | `HTTP Request Rate by Route/Status`, `SLO Burn Rate (5xx, 5m/1h)` | `BijuxAtlasHigh5xxRate` |
| `traffic-spike.md` | `HTTP Request Rate by Route/Status`, `Load Shedding Rate` | `BijuxAtlasOverloadSustained`, `BijuxAtlasP95LatencyRegression` |
| `slo-cheap-burn.md` | `SLO Burn Rate (5xx, 5m/1h)`, `SLO Error Budget Burn (cheap/standard)` | `BijuxAtlasCheapSloBurnFast`, `BijuxAtlasCheapSloBurnMedium`, `BijuxAtlasCheapSloBurnSlow` |
| `slo-standard-burn.md` | `SLO Burn Rate (5xx, 5m/1h)`, `SLO Error Budget Burn (cheap/standard)` | `BijuxAtlasStandardSloBurnFast`, `BijuxAtlasStandardSloBurnMedium`, `BijuxAtlasStandardSloBurnSlow` |
| `slo-overload-survival.md` | `Queue Depth and Overload`, `Load Shedding Rate` | `BijuxAtlasOverloadSurvivalViolated` |
| `slo-registry-refresh-stale.md` | `Catalog Refresh Duration` | `BijuxAtlasRegistryRefreshStale` |
| `slo-store-backend-error-spike.md` | `Store Open/Download p95`, `Store Error Spike Rate` | `BijuxAtlasStoreBackendErrorSpike` |

## Alert Coverage Registry

- `BijuxAtlasHigh5xxRate`
- `BijuxAtlasP95LatencyRegression`
- `BijuxAtlasStoreDownloadFailures`
- `BijuxAtlasCacheThrash`
- `AtlasOverloadSustained`
- `BijuxAtlasCheapSloBurnFast`
- `BijuxAtlasCheapSloBurnMedium`
- `BijuxAtlasCheapSloBurnSlow`
- `BijuxAtlasStandardSloBurnFast`
- `BijuxAtlasStandardSloBurnMedium`
- `BijuxAtlasStandardSloBurnSlow`
- `BijuxAtlasOverloadSurvivalViolated`
- `BijuxAtlasRegistryRefreshStale`
- `BijuxAtlasStoreBackendErrorSpike`

## Alert Drill Coverage Registry

- `alert-firing-proof`
- `cache-stampede`
- `cheap-endpoint-survival`
- `overload-admission-control`
- `registry-refresh-failure`
- `store-outage-under-load`

## Dashboard Panel Coverage Registry

- `Store Backend Fetch p95`
- `HTTP Request Rate by Route/Status`
- `HTTP p95 Latency by Route`
- `SQLite Query p95 by Class`
- `Store Open/Download p95`
- `Dataset Cache Hit/Miss`
- `Cache Hit Ratio`
- `Shed Rate by Reason`
- `Bulkhead Inflight by Class`
- `Bulkhead Saturation by Class`
- `Queue Depth and Overload`
- `Endpoint Request/Response Size p95`
- `Traffic Spike Drill View`
- `Rollout/Rollback View`
- `p99 Breakdown via Exemplars`
- `Dataset Cache Size`
- `SLO Burn Rate (5xx, 5m/1h)`
- `SLO Error Budget Burn (cheap/standard)`
- `SLO Health Status (cheap/standard)`

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
