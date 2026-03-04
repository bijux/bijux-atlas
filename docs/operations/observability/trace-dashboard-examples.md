# Trace Dashboard Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define practical trace dashboard views for runtime diagnosis.

## Dashboard panels

- Top routes by trace duration (p95).
- Slowest query spans grouped by dataset id.
- Shard routing span duration heatmap.
- Error-annotated request spans by route and status class.

## Drilldown filters

- `dataset_id`
- `query_type`
- `request_origin`
- `request_id`

## Validation

Use `make ops-trace-debug` and confirm at least one trace includes all filter fields above.
