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
- [Query benchmark dashboards](query-benchmark-dashboards.md): benchmark-focused latency and throughput panels
- [Metrics architecture](metrics-architecture.md): naming, labels, cardinality, and required runtime metrics
- [Metrics naming conventions](metrics-naming-conventions.md): canonical naming and unit suffix rules
- [Metric types](metric-types.md): counter, gauge, and histogram selection rules
- [Metric labels](metric-labels.md): stable label vocabulary
- [Cardinality policy](cardinality-policy.md): label cardinality controls
- [Metric retention policy](metric-retention-policy.md): retention tiers and budget alignment
- [Logging architecture](logging-architecture.md): structured schema, levels, and metadata policy
- [Logging debugging guide](logging-debugging-guide.md): request and query log triage flow
- [Logging ingestion examples](logging-ingestion-examples.md): parser and field mapping patterns
- [Logging aggregation examples](logging-aggregation-examples.md): incident-oriented aggregation slices
- [Logging dashboard examples](logging-dashboard-examples.md): operator panel ideas from logs
- [Logging best practices](logging-best-practices.md): stable authoring conventions
- [Log levels](log-levels.md): operational severity policy
- [Log metadata fields](log-metadata-fields.md): stable structured field contract
- [Log correlation IDs](log-correlation-ids.md): cross-signal request correlation flow
- [Log analysis workflows](log-analysis-workflows.md): repeatable triage flows
- [Logging sampling policy](logging-sampling-policy.md): deterministic log volume control
- [Logging redaction policy](logging-redaction-policy.md): masking and safe-field rules
- [Logging privacy policy](logging-privacy-policy.md): privacy constraints for runtime logs
- [Logging rotation policy](logging-rotation-policy.md): bounded local log retention controls
- [Observability lifecycle](../observability-lifecycle.md): how dashboards, alerts, and SLOs evolve safely
- [Observability setup](../observability-setup.md): minimum metrics, logs, and trace wiring
- [Tracing](tracing.md): trace-first diagnosis flow
- [Tracing architecture](tracing-architecture.md): runtime span model, propagation, and exporter setup
- [Tracing spans](tracing-spans.md): required span coverage surface
- [Trace fields](trace-fields.md): required trace identity and attributes
- [Trace exporters](trace-exporters.md): supported exporter modes and fallback
- [Trace sampling policy](trace-sampling-policy.md): sampling defaults and change rules
- [Trace timeline examples](trace-timeline-examples.md): healthy and failure span sequences
- [Trace dashboard examples](trace-dashboard-examples.md): practical dashboard panels and filters
- [Trace troubleshooting](trace-troubleshooting.md): exporter and propagation triage
- [Runtime diagnostics](runtime-diagnostics.md): debug endpoint contract and capture flow
- [Health endpoint semantics](health-endpoint-semantics.md): health and liveliness meaning
- [Readiness semantics](readiness-semantics.md): readiness gating behavior and status model
- [Diagnostics outputs](diagnostics-outputs.md): diagnostics endpoint output inventory
- [Debug commands](debug-commands.md): system debug command reference
- [Runtime introspection](runtime-introspection.md): cross-signal inspection workflow
- [Operational troubleshooting guide](operational-troubleshooting-guide.md): ordered incident diagnosis workflow
- [Production debugging guide](production-debugging-guide.md): safe runtime investigation procedure
- [Runtime inspection examples](runtime-inspection-examples.md): command examples for stable evidence capture
- [Failure analysis examples](failure-analysis-examples.md): common incident patterns and required checks
- [System state visualization](system-state-visualization.md): stable panel model for incident dashboards
- [Grafana dashboard examples](grafana-dashboard-examples.md): recommended panel families
- [Prometheus query examples](prometheus-query-examples.md): operational query snippets
- [Tracing dashboard examples](tracing-dashboard-examples.md): span and latency panel examples
- [Log analysis query examples](log-analysis-query-examples.md): repeatable log query snippets
- [Alert rule examples](alert-rule-examples.md): rule templates for core signals
- [Observability quickstart](observability-quickstart.md): shortest path to usable monitoring
- [Monitoring setup guide](monitoring-setup-guide.md): full setup sequence
- [Alerting configuration guide](alerting-configuration-guide.md): alert wiring and ownership
- [Production monitoring checklist](production-monitoring-checklist.md): go-live checks
- [Observability architecture diagram](observability-architecture-diagram.md): end-to-end telemetry flow
- [Observability FAQ](observability-faq.md): frequent operational decisions
- [Observability glossary](observability-glossary.md): canonical terminology
- [Observability troubleshooting guide](observability-troubleshooting-guide.md): failure-mode focused fixes
- [Observability performance considerations](observability-performance-considerations.md): runtime cost guardrails
- [Observability future roadmap](observability-future-roadmap.md): planned maturity upgrades
- [Cluster membership observability](cluster-membership-observability.md): metrics, traces, logs, and dashboards for node lifecycle
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
