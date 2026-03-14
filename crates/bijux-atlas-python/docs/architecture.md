# Python Client Architecture

## Goals

- Provide a stable Python API for Atlas dataset queries.
- Keep dependencies minimal and runtime behavior explicit.
- Support logging and trace correlation hooks from host applications.

## Components

- `ClientConfig`: validates base URL, timeout, and retry inputs.
- `AtlasClient`: user-facing query API.
- `HttpTransport`: JSON-over-HTTP transport with retry support.
- `RetryPolicy`: retry and backoff behavior.
- `Telemetry`: logging and tracing hook surface.
- `QueryRequest` and pagination helpers: request/response normalization.

## Request Flow

1. Application constructs `ClientConfig`.
2. `AtlasClient` validates config and builds `HttpTransport`.
3. `query()` serializes `QueryRequest` payload.
4. `HttpTransport` sends `POST /v1/query`, applies retry policy.
5. Response body is decoded and mapped into `Page`.

## Error Model

- `AtlasConfigError` for invalid configuration.
- `AtlasApiError` for server-side non-success responses.
- `AtlasRetryExhaustedError` for transport-level exhaustion.
