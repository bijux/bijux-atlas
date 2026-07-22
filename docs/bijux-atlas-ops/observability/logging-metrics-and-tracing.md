---
title: Logging, Metrics, and Tracing
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Logging, Metrics, and Tracing

Atlas uses three complementary signal types. Metrics establish population-level
impact, traces expose a request's execution path, and structured logs retain
event and policy context. Correlation—not volume—is what turns them into useful
operational evidence.

## Request Correlation

```mermaid
sequenceDiagram
    participant Client
    participant HTTP as Request boundary
    participant Work as Query or store work
    participant Telemetry as Telemetry pipeline
    Client->>HTTP: Request
    HTTP->>Telemetry: request_id, route, class, release
    HTTP->>Work: Propagate request and trace context
    Work->>Telemetry: spans, metrics, structured events
    HTTP-->>Client: Response
    Telemetry->>Telemetry: Join signals by time and identity
```

Request identifiers are required for request, query, and error spans and must
cross asynchronous boundaries. Logs must correlate with traces; traces must
correlate with request and latency metrics. Metrics intentionally do not carry
request or trace IDs because per-request labels would create unbounded series.

## Telemetry Pipeline Boundaries

```mermaid
flowchart LR
    Runtime[Runtime instrumentation] --> Buffer[Local buffer and exporter]
    Buffer --> Collector[Collector]
    Collector --> Metrics[Metrics backend]
    Collector --> Logs[Log backend]
    Collector --> Traces[Trace backend]
    Metrics --> Alert[Rules and notifications]
    Logs --> Investigate[Incident query]
    Traces --> Investigate
```

Instrumentation success does not establish backend ingestion. Backend
ingestion does not establish retention, queryability, alert evaluation, or
notification delivery. Monitor and drill each boundary required by the
operating claim.

## Structured Logs

Every governed log record carries `level`, `msg`, and `request_id`; registered
events also carry `event_name`. The event registry defines request start,
request end, policy rejection, cache lookup, store fetch, and SQLite query.
Request-end records include status and latency. Policy rejection records include
the policy, mode, reason, and limit.

Do not emit email, phone, IP address, social-security number, or personal name
fields. Keep high-cardinality request context in logs and traces, not metric
labels. See [Logging Contracts](logging-contracts.md) for the complete event
schema.

## Metrics

The metrics contract declares 39 required signals and their label sets. The
surface covers HTTP behavior, admission and shedding, cache use, registry age,
store requests and errors, dataset state, policy and invariant violations,
resource pressure, and request-stage latency.

Route, status, query type, stage, and error code are permitted dynamic
dimensions in the metric contract. Gene and transcript identifiers, raw names
and regions, IP addresses, request IDs, and trace IDs are forbidden. The
separate global cardinality policy caps the approved label vocabulary at 200
values; each metric also carries its own maximum-series and growth budget.

Use [Metrics Packages](metrics-packages.md) for the registry and golden scrape
surfaces.

## Traces

The endpoint contract assigns each of 15 routes to a cheap, medium, or heavy
class and names the spans required for that path. These include request root,
admission control, dataset resolution, cache lookup, store fetch, SQLite query,
and response serialization.

The tracing registry separately governs stable lifecycle identities for runtime,
HTTP, query, ingest, artifact, registry, configuration, startup, shutdown, and
structured-error spans. Stable trace identifiers are immutable; additions are
allowed, while a rename or deletion requires migration documentation. The two
layers answer different questions: endpoint spans prove request-path coverage,
while stable identities preserve longitudinal incident analysis.

Use [Tracing Pipelines](tracing-pipelines.md) for propagation and exporter
behavior.

## Diagnose with All Three Signals

1. Bound impact with request class, status, latency, saturation, and dependency
   metrics.
2. Select representative failing and successful traces from the same release
   and time window.
3. Correlate their request IDs with structured logs and policy events.
4. Compare dataset, release, and configuration identity before assigning cause.
5. Record sampling gaps, missing labels, or absent events as telemetry defects.

A dashboard can show correlation without establishing causation. Confirm the
fault through the governed contract, a controlled drill, or reproducible
request evidence before changing traffic or data.

## Signal Loss and Sampling

Record exporter failures, dropped events, queue saturation, scrape gaps, clock
skew, sampling policy, and retention limits with the observation window. A
missing trace may be sampling; a missing metric series may be instrumentation,
collection, or query failure. Classify the telemetry boundary before inferring
that the runtime event did not happen.

Sampling must preserve error and rare-path diagnostic value. Aggregate metrics
remain necessary for population impact because traces cannot be assumed to
represent the full request distribution.
