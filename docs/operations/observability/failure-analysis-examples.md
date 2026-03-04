# Failure Analysis Examples

## Saturation event

- Symptom: `/ready` returns `503`.
- Confirm with `/healthz/overload`.
- Inspect `/debug/runtime-stats` for queue depth and backlog growth.
- Inspect `/debug/query-planner-stats` for expensive plan pressure.

## Dataset routing anomaly

- Symptom: repeated dataset miss errors.
- Inspect `/debug/dataset-registry` for expected dataset visibility.
- Inspect `/debug/shard-map` and `/debug/cache-stats` for load/eviction churn.
