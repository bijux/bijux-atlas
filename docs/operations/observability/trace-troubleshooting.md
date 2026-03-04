# Trace Troubleshooting

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide deterministic triage steps when tracing output is missing or incomplete.

## Missing traces

1. Confirm `ATLAS_OTEL_ENABLED=true`.
2. Check `ATLAS_TRACE_EXPORTER` value is supported.
3. Verify exporter endpoint reachability (OTLP or Jaeger endpoint).
4. Run `make ops-traces-check`.

## Missing correlation fields

1. Send request with `x-request-id`, `x-correlation-id`, and `traceparent`.
2. Confirm response echoes IDs.
3. Confirm spans include `request_origin` and `dataset_id` where applicable.

## Exporter outage fallback

If remote exporter initialization fails, Atlas keeps local structured tracing enabled. Use file exporter for offline incident capture:

```bash
ATLAS_TRACE_EXPORTER=file ATLAS_TRACE_FILE_PATH=artifacts/logs/atlas-trace.jsonl
```
