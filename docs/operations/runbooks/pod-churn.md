# Pod Churn Runbook

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Symptoms

- Elevated 5xx during reschedules/evictions.
- Latency spikes during deployment churn.
- Readiness flaps across replicas.

## Metrics

- `bijux_http_requests_total` by status/route
- `bijux_http_request_latency_p95_seconds`
- `bijux_overload_shedding_active`
- pod restart count and rollout status

## Dashboards

- `docs/operations/observability/dashboard.md`
- Grafana URLs via `make ops-open-grafana`

## Commands

1. `make ops-drill-pod-churn`
2. `make ops-load-shedding`

## Expected outputs

- Cheap requests remain serviceable.
- Non-cheap requests may shed under pressure.
- Service returns to steady latency after churn.

## Mitigations

1. Increase replica floor and verify PDB/HPA settings.
2. Reduce rollout concurrency during high traffic.
3. Validate node-local cache profile for faster recovery.

## Alerts

- `BijuxAtlasHigh5xxRate`
- `BijuxAtlasP95LatencyRegression`

## Rollback

1. Roll back rollout settings to previous stable values.
2. Re-run `make ops-drill-pod-churn` to confirm stabilization.

## Postmortem checklist

- Capture `make ops-report` artifacts and pod events for the drill window.
- Record peak latency and error-rate deltas during churn.
- Track whether cheap endpoints stayed available throughout the drill.
