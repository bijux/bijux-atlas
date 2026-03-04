# Tracing Architecture

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define runtime tracing architecture, propagation, sampling, and exporter configuration.

## Architecture

Atlas tracing is span-based and request-centric:

1. inbound request span (`http.request`)
2. admission and policy spans
3. query planning and execution spans
4. cache, shard routing, and artifact loading spans
5. serialization and response completion spans

## Trace initialization

Tracing initialization is centralized in `crates/bijux-atlas-server/src/telemetry/tracing.rs` and configured through runtime config.

## Runtime configuration

- `ATLAS_OTEL_ENABLED`
- `ATLAS_TRACE_EXPORTER` (`otlp`, `jaeger`, `file`, or `none`)
- `ATLAS_TRACE_SAMPLING_RATIO` (`0.0..=1.0`)
- `ATLAS_TRACE_OTLP_ENDPOINT` (optional)
- `ATLAS_TRACE_JAEGER_ENDPOINT` (optional; defaults to local OTLP ingest endpoint)
- `ATLAS_TRACE_FILE_PATH` (optional; local JSONL trace sink when exporter is `file`)
- `ATLAS_TRACE_SERVICE_NAME`
- `ATLAS_TRACE_CONTEXT_PROPAGATION_ENABLED`

## Propagation model

- request id propagates via `x-request-id`
- correlation id propagates via `x-correlation-id`
- trace context propagates via `traceparent`

## Required span coverage

- HTTP request
- query planning
- query execution
- ingest pipeline
- dataset loading
- cache lookup
- shard routing
- artifact loading
- cursor generation
- API serialization

## Exporter behavior

- `otlp`: exports spans to the configured OTLP endpoint.
- `jaeger`: exports via OTLP endpoint compatible with Jaeger collector ingest.
- `file`: writes line-delimited JSON spans to local path for offline debugging.
- `none`: keeps local structured tracing without remote span export.

If remote exporter initialization fails, Atlas falls back to local tracing output so runtime diagnostics remain available.
