# Tracing and Correlation

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## What

Defines the request span chain and correlation model.

## Why

Makes latency decomposition and failure diagnosis deterministic.

## Contracts

Request path spans in sequence:

- `admission_control`
- `dataset_resolve`
- `cache_lookup`
- `store_fetch` (cache miss path)
- `open_db`
- `sqlite_query`
- `serialize_response`

## Request Correlation
- Every request receives `x-request-id` response header.
- Server logs include `request_id` so logs can be joined with metrics/traces.

## OpenTelemetry

Atlas uses `tracing` instrumentation points compatible with OpenTelemetry subscribers.
To export spans, attach a `tracing-opentelemetry` layer in server startup.

## Exemplars
If `ApiConfig.enable_exemplars=true`, `/metrics` emits a last-seen request id exemplar comment:

`# atlas_exemplar_last_request_id req-...`

This can be wired to trace lookup in collector-side relabeling.

## Failure modes

Missing spans or request IDs break trace-to-metric correlation during incident triage.

## How to verify

```bash
make ops-traces-check
make ops-observability-validate
```

## See also

- `ops-ci`
