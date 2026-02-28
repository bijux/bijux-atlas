# Dashboards

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define the dashboard order for incident triage.

## Triage order

1. Service health dashboard: availability, error rate, p95/p99 latency.
2. Store dashboard: query latency, backend errors, saturation.
3. Dataset pipeline dashboard: ingest failures and promotion lag.
4. Platform dashboard: pod churn, resource pressure, restart rate.

## Verify success

```bash
make ops-observability-verify
```

Expected result: all required dashboard queries resolve and chart panels load.

## Next

- [Tracing](tracing.md)
- [Incident Response](../incident-response.md)
