# Runbook: Incident Playbook

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Symptoms

- Any sustained SLO breach or elevated 5xx/timeout class errors.

## Metrics

- `bijux_http_requests_total`
- `bijux_http_request_latency_p95_seconds`
- `bijux_errors_total`

## Dashboard Panels

- `HTTP Request Rate by Route/Status`
- `HTTP p95 Latency by Route`
- `SLO Burn Rate (5xx, 5m/1h)`
- `Dataset Cache Hit/Miss`

## Commands

```bash
$ curl -s http://127.0.0.1:8080/healthz
$ curl -s http://127.0.0.1:8080/readyz
$ curl -s http://127.0.0.1:8080/metrics
```

## Expected outputs

- Health/ready reflect actual availability.
- Metrics expose route/status/error trends needed for triage.

## Mitigations

- Apply class-based shedding and rate controls.
- Shift to cached-only mode when store outage is primary cause.

## Alerts

- `BijuxAtlasHigh5xxRate`
- `BijuxAtlasP95LatencyRegression`

## Rollback

- Roll back last deployment/config release if issue is rollout-induced.

## Postmortem checklist

- Customer impact quantified.
- Root cause and trigger captured.
- Follow-up tasks linked to owners.

## See also

- `ops-ci`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
