# API troubleshooting

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: map common API symptoms to checks and likely fixes.

## Troubleshooting map

| Symptom | Check | Fix |
| --- | --- | --- |
| Request fails with `MissingDatasetDimension` | Confirm `release`, `species`, and `assembly` are all present | Resend the request with full dataset identity |
| Request fails with `InvalidCursor` | Compare the current query string to the query that produced the cursor | Restart pagination from page one for the new query |
| Request fails with `QueryRejectedByPolicy` or `ResponseTooLarge` | Review `limit`, `range`, and optional includes | Narrow the request or use `POST /v1/query/validate` before retrying |
| Request fails with `NotReady` | Probe `GET /readyz` | Wait for readiness or involve operators if the condition persists |
| Repeated `429` or `Timeout` responses | Inspect your client retry rate and concurrency | Slow down, add jitter, and follow [Client retries and backoff](client-retries-and-backoff.md) |

## Verification

```bash
curl -i -fsS 'http://127.0.0.1:8080/readyz'
```

Expected output: HTTP `200` when the service is ready, or HTTP `503` with a stable error body when it is not.

## Next steps

- [Errors](errors.md)
- [Operations incident response](../operations/incident-response.md)
