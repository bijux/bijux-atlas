---
title: Tracing Pipelines
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Tracing Pipelines

Tracing support is declared through OpenTelemetry collector inputs and Atlas
trace verification paths rather than as undocumented runtime behavior.

## Purpose

Use this page to understand how traces should flow from runtime spans through
the collector into storage and visualization, and what must be validated before
trace coverage can be trusted.

## Source of Truth

- `ops/observe/tracing/architecture.json`
- `ops/observe/tracing/correlation-policy.json`
- `ops/observe/tracing/span-registry.json`
- `ops/observe/contracts/trace-structure.golden.json`
- `ops/observe/pack/compose/otel-collector.yaml`
- `ops/observe/pack/otel/config.yaml`

## End-to-End Tracing Path

Atlas tracing is expected to follow this path:

1. runtime emits required spans such as `http.request`, `query.execution`,
   `artifact.load`, `registry.access`, and lifecycle spans
2. correlation policy propagates identifiers like `request_id` across async
   boundaries
3. the OpenTelemetry collector configuration receives and forwards the spans
4. dashboards or trace backends make the data usable for operators

## Verification Expectations

Trust the tracing pipeline only when all of these hold:

- the span registry still matches the implemented trace surface
- the trace structure golden requirements still pass
- logs and metrics can be correlated to the same request path
- collector configuration still matches the declared tracing architecture

## Related Contracts and Assets

- `ops/observe/tracing/`
- `ops/stack/otel/`
- `ops/observe/pack/compose/otel-collector.yaml`
