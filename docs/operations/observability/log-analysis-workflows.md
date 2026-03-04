# Log Analysis Workflows

## Error spike triage

1. Filter `level=error` by time window.
2. Group by `event_id` and `route`.
3. Pivot on `status` and `dataset_id`.

## Timeout triage

1. Filter `event_id` for timeout/retry signals.
2. Correlate with queue depth and slow query metrics.
3. Validate affected shard and dataset coverage.

## Policy rejection triage

1. Filter policy rejection `event_id`s.
2. Group by `query_type` and `route`.
3. Confirm rule intent before tuning limits.
