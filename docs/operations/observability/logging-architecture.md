# Logging Architecture

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@b77ca37b3`
- Reason to exist: define structured runtime logging model, schema, and metadata conventions.

## Logging philosophy

Atlas logging is structured-first: every operationally relevant event is emitted as machine-parseable fields with stable `event_id` values.

## Log schema

Canonical schema: `configs/contracts/observability/log.schema.json`.

Required fields:

- `timestamp`
- `level`
- `target`
- `fields.message`

Common metadata fields:

- `event_id`
- `request_id`
- `route`
- `status`
- `dataset_id`
- `query_type`
- `shard_id`

## Level policy

- `trace`: narrow debugging sessions
- `debug`: temporary diagnosis of non-prod incidents
- `info`: normal runtime lifecycle and request flow
- `warn`: degraded behavior, retries, and timeouts
- `error`: hard failures and rejected operations

## Format and configuration

- JSON output: `ATLAS_LOG_JSON=true`
- Level: `ATLAS_LOG_LEVEL`
- Target filtering: `ATLAS_LOG_FILTER_TARGETS`
- Sampling: `ATLAS_LOG_SAMPLING_RATE`
- Redaction toggle: `ATLAS_LOG_REDACTION_ENABLED`
- Rotation bytes: `ATLAS_LOG_ROTATION_MAX_BYTES`
- Rotation files: `ATLAS_LOG_ROTATION_MAX_FILES`

## Next

- [Logging debugging guide](logging-debugging-guide.md)
- [Logging best practices](logging-best-practices.md)
- [Logging sampling policy](logging-sampling-policy.md)
