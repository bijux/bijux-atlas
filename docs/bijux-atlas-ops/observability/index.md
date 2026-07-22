---
title: Observability
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Observability

Atlas observability is an operating system for decisions, not a collection of
charts. Runtime signals are joined to endpoint contracts, cardinality limits,
alert ownership, drills, and evidence records so an operator can move from a
symptom to a bounded action.

## Signal-to-Decision Model

```mermaid
flowchart LR
    R["Request and lifecycle events"] --> L["Structured logs"]
    R --> M["Governed metrics"]
    R --> T["Correlated traces"]
    M --> A["Alert contract"]
    L --> I["Incident diagnosis"]
    T --> I
    A --> I
    I --> D{"Operating decision"}
    D -->|protect| S["Shed or isolate work"]
    D -->|recover| C["Restart, rollback, or restore"]
    D -->|promote| E["Preserve release evidence"]
```

No signal is authoritative by itself. Metrics quantify scope, traces localize a
request path, and logs explain discrete events and policy decisions. Release,
dataset, profile, and request-class identity must remain visible across them.

## Correlation Spine

```mermaid
flowchart TD
    Request[Request or lifecycle event] --> RequestId[Request and trace identity]
    Request --> ReleaseId[Runtime and dataset release identity]
    Request --> Context[Profile, instance, route, and request class]
    RequestId --> Logs[Structured logs]
    RequestId --> Traces[Trace and spans]
    ReleaseId --> Metrics[Metrics and SLO windows]
    Context --> Metrics
    Logs --> Evidence[Correlated evidence window]
    Traces --> Evidence
    Metrics --> Evidence
```

Correlation fields should be stable enough to join signals without putting
high-cardinality values into metric labels. Request and trace identifiers
belong in logs and traces. Metrics carry bounded dimensions such as route,
request class, result class, profile, and release identity where their
contracts permit them.

## Signal Quality

| Property | Question |
| --- | --- |
| coverage | Does every protected path emit the signals named by its contract? |
| freshness | Is collection delay short enough for the decision window? |
| continuity | Are gaps, restarts, and exporter loss visible? |
| correlation | Can metrics, logs, traces, release, and dataset identity be joined? |
| cardinality | Are dimensions bounded so the telemetry system survives load? |
| retention | Will raw evidence outlive the incident or promotion review? |
| redaction | Are secrets and sensitive payloads excluded without removing decision identity? |

## End-to-End Signal Assurance

```mermaid
flowchart LR
    Instrument[Runtime instrumented] --> Emit[Signal emitted]
    Emit --> Transport[Exporter or scrape succeeds]
    Transport --> Store[Backend ingests and retains]
    Store --> Query[Operator query returns the window]
    Query --> Evaluate[Rule or SLO evaluates]
    Evaluate --> Notify[Notification reaches its owner]
    Notify --> Act[Decision is recorded]
```

Each arrow is a separate availability boundary. A unit test can establish
instrumentation shape, but not exporter delivery. A healthy collector can
establish transport, but not backend retention. A firing rule can establish
evaluation, but not notification delivery or ownership response.

| Assurance level | Evidence | Claim supported |
| --- | --- | --- |
| Declared | Registry, rule, dashboard, or drill definition exists and validates. | The intended signal and owner are specified. |
| Emitted | A known request or lifecycle event produces the expected runtime signal. | Instrumentation is active for that path. |
| Delivered | Backend query returns the signal with expected labels and timestamp. | Transport, ingestion, and retention worked for the observed window. |
| Exercised | A controlled condition produces the rule, notification, dashboard, and trace/log correlation. | The operating path worked for the named environment and drill. |
| Retained | Immutable evidence binds the exercised path to release, dataset, profile, and time. | Another operator can reconstruct the decision after live telemetry expires. |

Do not collapse these levels into “observability present.” Promotion evidence
should name the highest level actually demonstrated for every required signal
path.

## Choose the Operating Question

| Question | Start here | Decision supported |
| --- | --- | --- |
| Is the service safe to receive traffic? | [Health, Readiness, and Drain](health-readiness-and-drain.md) | Admit, drain, or remove an instance. |
| Which runtime path is failing? | [Logging, Metrics, and Tracing](logging-metrics-and-tracing.md) | Localize request, store, cache, or policy behavior. |
| What must a structured event contain? | [Logging Contracts](logging-contracts.md) | Validate event identity, required fields, and data handling. |
| Which metric and labels are governed? | [Metrics Packages](metrics-packages.md) | Validate metric ownership and cardinality. |
| How does trace context move? | [Tracing Pipelines](tracing-pipelines.md) | Validate propagation, correlation, and exporter behavior. |
| Does a signal require action? | [Alert Rules](alert-rules.md) | Page, investigate, or monitor. |
| Which view explains the impact? | [Dashboards and Panels](dashboards-and-panels.md) | Correlate service, dependency, and saturation signals. |
| Can the telemetry path survive failure? | [Telemetry Drills](telemetry-drills.md) | Accept or reject monitoring readiness. |
| What must accompany a decision? | [Operational Evidence Reports](operational-evidence-reports.md) | Preserve reproducible incident or release evidence. |

## Contracted Surface

The checked-in contracts currently define:

- 15 HTTP endpoints with request classes, required metrics, and path spans;
- 39 required metrics with label sets, owners, semantics, and cardinality
  budgets;
- structured log fields and registered request and policy events;
- six required request-path spans plus ten stable lifecycle span identities;
- 20 governed alert specifications tied to an owner, runbook, drill, and
  invariant; and
- two telemetry-path drills for an OpenTelemetry outage and Prometheus gap.

The generated telemetry index inventories the alert catalog, dashboard,
readiness, SLO, and drill artifacts. It proves that those assets are discoverable;
it does not prove that a deployed collector, rule engine, or notification path
is working. Use drills and captured runtime evidence for that claim.

## Investigation Discipline

Preserve the observation window, release and dataset identities, selected
profile, alert and rule versions, dashboard snapshot, representative trace IDs,
and relevant structured logs. Treat missing telemetry as a finding: an incident
cannot be declared understood when the evidence needed to distinguish runtime,
store, catalog, or policy failure is absent.

Telemetry degradation changes the decision boundary. A serving runtime may
remain available while promotion is held because alert delivery, trace
retention, or required measurements cannot be established.

When one signal type is missing, use the remaining signals to bound impact but
record the blind spot. Metrics without traces can show population harm without
localizing a request path. Traces without metrics can explain examples without
establishing prevalence. Logs without a reliable window can retain events
without proving what was absent.

For incident execution, continue to [Incident Response](incident-response.md).
