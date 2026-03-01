# Dashboards

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define the dashboard order for incident triage.

## What to watch

1. Service health dashboard: availability, error rate, p95 and p99 latency, and request volume.
2. Store dashboard: query latency, backend errors, saturation, cache pressure, and slow reads.
3. Dataset pipeline dashboard: ingest failures, promotion lag, and dataset freshness.
4. Platform dashboard: pod churn, resource pressure, restart rate, and node health.

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
