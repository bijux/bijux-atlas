# Log Schema Documentation

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: document the canonical log schema used by logging validation and analysis.

## Contract sources

- `configs/contracts/observability/log.schema.json`
- `ops/observe/contracts/logs-fields-contract.json`
- `ops/observe/logging/format-validator-contract.json`

## Required fields

- `timestamp`
- `level`
- `target`
- `message`
- `request_id`
- `event_name`

## Correlation fields

- `request_id` (required)
- `trace_id` (optional but expected for traced request paths)
