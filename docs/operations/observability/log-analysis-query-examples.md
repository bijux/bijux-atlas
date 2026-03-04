# Log Analysis Query Examples

Top error event ids in last 15 minutes:

```text
level:error | stats count() by event_id | sort -count
```

Timeouts by route:

```text
event_id:timeout OR event_id:retry_timeout | stats count() by route
```

Policy rejections by query type:

```text
event_id:query_rejected_by_policy | stats count() by query_type, route
```
