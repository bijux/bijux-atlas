# Observability

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: provide the single observability entrypoint for detection, diagnosis, and recovery verification.

## Purpose

Detect service regression quickly, route alerts to actionable runbooks, and confirm recovery.

## What you will find here

- [Alerts](alerts.md): alert-to-runbook routing and severity model
- [Dashboards](dashboards.md): dashboard set for incident triage
- [Metrics architecture](metrics-architecture.md): naming, labels, cardinality, and required runtime metrics
- [Observability lifecycle](../observability-lifecycle.md): how dashboards, alerts, and SLOs evolve safely
- [Observability setup](../observability-setup.md): minimum metrics, logs, and trace wiring
- [Tracing](tracing.md): trace-first diagnosis flow
- [SLO policy](slo-policy.md): target objectives and burn policy
- [SLOs with PromQL](slos-with-promql.md): practical query patterns for burn analysis
- Alert rule source: `ops/observe/alerts/atlas-alert-rules.yaml`
- Dashboard source: `ops/observe/dashboards/atlas-observability-dashboard.json`
- Contract reference: [Observability Contracts](../../reference/contracts/observability.md)

## Verify success

```bash
make ops-observability-verify
```

Expected result: alert, metric, and trace checks pass for the current environment.

## Governed interfaces

- Metrics must satisfy `configs/contracts/observability/metrics.schema.json`.
- Structured logs must satisfy `configs/contracts/observability/log.schema.json`.
- Error codes must stay aligned with `configs/contracts/observability/error-codes.json`.
- Release evidence includes the observability assets used for the current candidate bundle.

## Next

- [Incident Response](../incident-response.md)
- [Runbooks](../runbooks/index.md)
