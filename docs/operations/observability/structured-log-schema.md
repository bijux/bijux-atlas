# Structured Log Schema

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide the contract for structured logging records.

## Required fields

- `timestamp`
- `level`
- `target`
- `message`
- `request_id`
- `event_name`

## Optional fields

- `trace_id`
- `route`
- `status`
- `dataset_id`
- `query_type`
- `shard_id`

## Validation

- Accepts log levels: `TRACE|DEBUG|INFO|WARN|ERROR` (case-insensitive).
- Rejects redaction blocklist patterns in the `message` field.
- Requires event naming prefixes that map to known classes (`runtime_`, `query_`, `ingest_`, `artifact_`, `config_`, `startup_`, `shutdown_`, `security_`).
