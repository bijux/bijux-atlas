# Logging Ingestion Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@053b86165`
- Reason to exist: show structured log ingestion patterns for downstream systems.

## Example filters

- Keep events with `event_id`.
- Parse nested `fields` keys into indexable attributes.
- Preserve `request_id`, `query_id`, and `dataset_id` as searchable fields.
