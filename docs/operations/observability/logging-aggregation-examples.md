# Logging Aggregation Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define practical log aggregation slices for incident triage.

## Aggregation slices

- Error count by `route` and `event_id`.
- Timeout count by `dataset_id`.
- Retry volume by backend and status class.
- Slow query warnings by `query_id`.
