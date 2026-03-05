# Log Output Examples

## Request completion

```json
{"timestamp":"2026-03-05T10:00:00Z","level":"INFO","target":"atlas::runtime","message":"request completed","request_id":"req-123","trace_id":"trace-123","event_name":"query_execute"}
```

## Ingest rejection with redaction-safe text

```json
{"timestamp":"2026-03-05T10:00:01Z","level":"WARN","target":"atlas::ingest","message":"ingest rejected by policy","request_id":"req-124","event_name":"ingest_policy_reject"}
```
