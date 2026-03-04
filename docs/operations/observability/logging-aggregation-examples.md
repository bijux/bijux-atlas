# Logging Aggregation Examples

- Owner: `bijux-atlas-operations`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@053b86165`
- Reason to exist: define practical log aggregation slices for incident triage.

## Aggregation slices

- Error count by `route` and `event_id`.
- Timeout count by `dataset_id`.
- Retry volume by backend and status class.
- Slow query warnings by `query_id`.
