# Log Metadata Fields

Required stable fields:

- `event_id`
- `request_id`
- `route`
- `status`
- `dataset_id`
- `query_type`
- `shard_id`

Rule:

- Keep field names stable across releases so queries and alerts do not break.
