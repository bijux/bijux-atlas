# Observability setup

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define the minimal metrics, logs, and traces setup required before Atlas is considered operable.

## Required signals

- metrics scraping for service and store health
- structured logs for startup, deploy, and error diagnosis
- traces that link API, query, and store spans

## Setup flow

1. configure telemetry endpoints in deploy values
2. render and inspect the install plan
3. apply the deployment
4. verify metrics, logs, and traces with the canonical wrapper

## Verify success

```bash
make ops-observability-install
make ops-observability-verify
```

Expected outputs:

- telemetry wiring appears in the plan
- verify command confirms metrics, logs, and traces are reachable

## Rollback

If telemetry wiring breaks deploy safety, revert the rollout with [Rollback procedure](release/rollback-procedure.md).

## Next steps

- [Dashboards](observability/dashboards.md)
- [Alerts](observability/alerts.md)
- [Tracing](observability/tracing.md)
