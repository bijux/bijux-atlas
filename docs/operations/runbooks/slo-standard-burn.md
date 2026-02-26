# SLO Standard Burn

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Symptoms

- `BijuxAtlasStandardSloBurnFast|Medium|Slow` firing.
- Increased 5xx ratio on `class="standard"` routes.

## Metrics

- `http_requests_total{class="standard",status=~"5.."}`
- `http_requests_total{class="standard"}`
- `http_request_duration_seconds_bucket{class="standard"}`

## Commands

```bash
make ops-slo-alert-proof
kubectl -n atlas-e2e get pods
```

## Expected outputs

- Standard-burn alert windows are visible and recover below threshold.
- Standard request success ratio stabilizes within SLO objective.

## Mitigations

- Scale API replicas and validate readiness/liveness stability.
- Inspect recent deploy/config drift affecting standard routes.
- Throttle heavy workloads to preserve standard request budget.

## Alerts

- Primary alerts: `BijuxAtlasStandardSloBurnFast`, `BijuxAtlasStandardSloBurnMedium`, `BijuxAtlasStandardSloBurnSlow`.
- Dashboard: `docs/operations/observability/dashboard.md`.
- Drill reference: `make ops-drill-overload`.

## Rollback

- Roll back to previous release/profile proven by `ops-k8s-tests`.

## Postmortem checklist

- Document impacted endpoints, peak burn multiplier, and owner action items.
