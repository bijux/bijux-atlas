# Tracing and Correlation

## Span Chain
Request path is traced with spans in this sequence:

1. `admission_control`
2. `dataset_resolve`
3. `cache_lookup`
4. `store_fetch` (cache miss path)
5. `open_db`
6. `sqlite_query`
7. `serialize_response`

## Request Correlation
- Every request receives `x-request-id` response header.
- Server logs include `request_id` so logs can be joined with metrics/traces.

## OpenTelemetry
Atlas uses `tracing` instrumentation points compatible with OpenTelemetry subscribers.
To export OTEL spans, attach a `tracing-opentelemetry` layer in server startup.

## Exemplars
If `ApiConfig.enable_exemplars=true`, `/metrics` emits a last-seen request id exemplar comment:

`# atlas_exemplar_last_request_id req-...`

This can be wired to trace lookup in collector-side relabeling.

## See also

- `ops-ci`
