# SLO Store Backend Error Spike

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Symptoms

- `BijuxAtlasStoreBackendErrorSpike` firing.
- Rapid increase in store errors impacting standard/heavy traffic.

## Metrics

- `atlas_store_errors_total{backend}`
- `http_requests_total{class=~"standard|heavy"}`
- `atlas_shed_total{reason}`

## Commands

```bash
make ops-drill-store-outage
make ops-drill-toxiproxy-latency
```

## Expected outputs

- Store error ratio falls below threshold.
- API returns degrade gracefully with controlled shedding.

## Mitigations

- Inspect store backend health and network path latency.
- Fail over to healthy backend/profile if available.
- Reduce heavy request load until backend recovers.

## Alerts

- Primary alert: `BijuxAtlasStoreBackendErrorSpike`.
- Dashboard: `docs/operations/observability/dashboard.md`.
- Drill references: `make ops-drill-store-outage`, `make ops-drill-toxiproxy-latency`.

## Rollback

- Roll back store client config/version and retry policy changes.

## Postmortem checklist

- Capture backend error spike source, blast radius, and corrective actions.
