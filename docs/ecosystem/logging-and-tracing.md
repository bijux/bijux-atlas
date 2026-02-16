# Bijux Logging And Tracing Standard

## Logging Format

- Default log format is JSON for all runtime services.
- Human-text logs may be enabled only with explicit local override.
- Required structured fields:
  - `timestamp`
  - `level`
  - `subsystem`
  - `version`
  - `request_id`

## Trace ID Propagation

- Incoming `x-request-id` is accepted and propagated unchanged.
- If missing, `traceparent` is used as source for request correlation.
- If both are missing, service generates a deterministic request id.
- Response must include `x-request-id` header.
