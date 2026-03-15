---
title: Logging, Metrics, and Tracing
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Logging, Metrics, and Tracing

Observability is what turns Atlas from a black box into an operable service.

## Observability Layers

```mermaid
flowchart LR
    Requests[Requests] --> Logs[Logs]
    Requests --> Metrics[Metrics]
    Requests --> Traces[Traces]
    Logs --> Diagnosis[Diagnosis]
    Metrics --> Diagnosis
    Traces --> Diagnosis
```

## What Each Signal Is Good For

- logs explain events and failures in context
- metrics show aggregate runtime behavior and saturation trends
- traces help follow a request path across internal work

## Metrics Surface

```mermaid
flowchart TD
    Runtime[Runtime] --> MetricsEndpoint[/metrics]
    MetricsEndpoint --> Scrape[Prometheus-style scraping]
    Scrape --> Alerting[Dashboards and alerts]
```

## Operational Priorities

When observing Atlas, pay closest attention to:

- readiness and overload behavior
- request classification and rejection patterns
- cache and store latency patterns
- request rate, concurrency, and error trends

## Logging Practice

- keep logs structured and machine-parseable where possible
- use request correlation data during incident analysis
- prefer stable fields and identifiers over ad hoc human prose only

## Tracing Practice

- use traces when request-level latency or path ambiguity matters
- correlate tracing with metrics rather than treating either as sufficient alone

