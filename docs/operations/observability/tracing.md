# Tracing

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: provide a deterministic trace-first diagnosis path.

## Trace-first diagnosis

1. Start from alert timestamp and capture a 15-minute window.
2. Identify top failing route by error count.
3. Find the slowest span group for the failing route.
4. Correlate span IDs with store and dataset operations.
5. Select mitigation from the mapped runbook.

## Verify success

```bash
make ops-observability-verify
```

Expected result: traces can be filtered by route and correlated to failing component spans.

## Next

- [Alerts](alerts.md)
- [Incident Response](../incident-response.md)
