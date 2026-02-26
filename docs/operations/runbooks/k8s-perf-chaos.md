# Runbook: K8s Perf Chaos

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Symptoms

- p99 spikes during pod churn/noisy-neighbor scenarios.

## Metrics

- `bijux_http_request_latency_p95_seconds`
- `bijux_overload_shedding_active`
- `bijux_dataset_hits`

## Commands

```bash
$ k6 run ops/load/k6/suites/warm-steady.js
$ kubectl delete pod -n default -l app.kubernetes.io/name=bijux-atlas --force --grace-period=0
```

## Expected outputs

- Cheap queries remain available.
- Shedding toggles for heavy classes during turbulence.

## Mitigations

- Increase min replicas and tune bulkheads.
- Cap warmup/download concurrency.

## Alerts

- `BijuxAtlasP95LatencyRegression`
- `BijuxAtlasHigh5xxRate`

## Rollback

- Revert recent HPA or cache policy changes.

## Postmortem checklist

- Chaos scenario report archived.
- Regression threshold updates reviewed.
- Runbook improvements captured.

## See also

- `ops-ci`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
