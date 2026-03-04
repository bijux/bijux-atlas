# Trace Exporters

Supported exporters:

- `otlp`
- `jaeger` (via OTLP ingest path)
- `file`
- `none`

Configuration:

- `ATLAS_TRACE_EXPORTER`
- `ATLAS_TRACE_OTLP_ENDPOINT`
- `ATLAS_TRACE_JAEGER_ENDPOINT`
- `ATLAS_TRACE_FILE_PATH`

Fallback rule:

- On remote exporter failure, keep local trace output available for incident diagnostics.
