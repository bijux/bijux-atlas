# Logging Ingestion Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: show structured log ingestion patterns for downstream systems.

## Example filters

- Keep events with `event_id`.
- Parse nested `fields` keys into indexable attributes.
- Preserve `request_id`, `query_id`, and `dataset_id` as searchable fields.
