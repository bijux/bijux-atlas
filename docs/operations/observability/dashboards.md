# Dashboards

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the dashboard order for incident triage.

## What to watch

1. Service health dashboard: availability, error rate, p95 and p99 latency, and request volume.
2. Store dashboard: query latency, backend errors, saturation, cache pressure, and slow reads.
3. Dataset pipeline dashboard: ingest failures, promotion lag, and dataset freshness.
4. Platform dashboard: pod churn, resource pressure, restart rate, and node health.

## Dashboard artifacts

- `ops/observe/dashboards/atlas-runtime-health-dashboard.json`
- `ops/observe/dashboards/atlas-query-performance-dashboard.json`
- `ops/observe/dashboards/atlas-ingest-pipeline-dashboard.json`
- `ops/observe/dashboards/atlas-artifact-registry-dashboard.json`
- `ops/observe/dashboards/atlas-system-resources-dashboard.json`
- `ops/observe/dashboards/atlas-slo-compliance-dashboard.json`
- `ops/observe/dashboards/atlas-error-classification-dashboard.json`
- `ops/observe/dashboards/atlas-latency-distribution-dashboard.json`
- `ops/observe/dashboards/atlas-artifact-cache-performance-dashboard.json`
- `ops/observe/dashboards/atlas-drift-detection-dashboard.json`

## Triage order

1. Start with service health to confirm whether the incident is user-visible.
2. Move to store or dataset views to identify the failing subsystem.
3. Use platform views only after a service or store symptom points to infrastructure pressure.

## Verify success

```bash
make ops-observability-verify
make ops-dashboards-validate
```

Expected result: all required dashboard queries resolve and chart panels load.

## Rollback

If a dashboard change removes required panels or queries, revert the dashboard pack change and rerun validation.

## Next

- [Tracing](tracing.md)
- [Incident Response](../incident-response.md)
