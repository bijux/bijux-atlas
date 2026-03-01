# SLO policy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define operator SLO targets, representative query shapes, and the burn-response workflow.

## Targets

- availability target: keep the main read path within the published service objective
- latency targets: watch p50, p95, and p99 for the user-facing read path
- error target: keep backend and API error ratios below the burn threshold

## Representative query shapes

- cheap read: common gene lookup with explicit dataset dimensions
- medium read: paginated list calls with filters and cursors
- expensive read: broader range or diff-style queries that still remain inside policy

## Burn response

1. confirm the alert in [Alerts](alerts.md)
2. inspect dashboards and traces for the dominant failing route
3. move to the mapped runbook if the burn is sustained

## Verify success

```bash
make ops-slo-report
make ops-slo-burn
```

Expected result: SLO evidence renders and burn logic resolves to the expected operator action.

## Rollback

If an SLO rule change causes false paging or hides real burn, revert the rule change and rerun the SLO reports.

## Next

- [Alerts](alerts.md)
- [Dashboards](dashboards.md)
- [Tracing](tracing.md)
