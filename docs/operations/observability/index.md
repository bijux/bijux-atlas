# Observability

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: provide the single observability entrypoint for detection, diagnosis, and recovery verification.

## Purpose

Detect service regression quickly, route alerts to actionable runbooks, and confirm recovery.

## What you will find here

- [Alerts](alerts.md): alert-to-runbook routing and severity model
- [Dashboards](dashboards.md): dashboard set for incident triage
- [Observability setup](../observability-setup.md): minimum metrics, logs, and trace wiring
- [Tracing](tracing.md): trace-first diagnosis flow
- [SLO policy](slo-policy.md): target objectives and burn policy
- [SLOs with PromQL](slos-with-promql.md): practical query patterns for burn analysis
- Alert rule source: `ops/observe/alerts/atlas-alert-rules.yaml`

## Verify success

```bash
make ops-observability-verify
```

Expected result: alert, metric, and trace checks pass for the current environment.

## Next

- [Incident Response](../incident-response.md)
- [Runbooks](../runbooks/index.md)
