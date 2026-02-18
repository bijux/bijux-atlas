# API Errors

- Owner: `api`
- Stability: `stable`

## Envelope

All API errors use:

```json
{
  "error": {
    "code": "InvalidQueryParameter",
    "message": "invalid query parameter: limit",
    "details": {
      "field_errors": [
        {"parameter": "limit", "reason": "invalid", "value": "0"}
      ]
    },
    "request_id": "req-0000000000000001"
  }
}
```

`hint` and `retryable` are stable fields carried in `error.details` when available.
`request_id` is always present at top-level and mirrored in `X-Request-Id`.

## HTTP Mapping

| HTTP | Codes |
| --- | --- |
| 400 | `InvalidQueryParameter`, `MissingDatasetDimension`, `InvalidCursor`, `ValidationFailed` |
| 404 | `DatasetNotFound`, `GeneNotFound` |
| 409 | `ArtifactCorrupted`, `ArtifactQuarantined` |
| 413 | `PayloadTooLarge`, `ResponseTooLarge`, `RangeTooLarge` |
| 422 | `QueryRejectedByPolicy`, `QueryTooExpensive`, ingest validation codes |
| 429 | `RateLimited` |
| 503 | `NotReady`, `UpstreamStoreUnavailable` |
| 504 | `Timeout` |
| 500 | `Internal` |

For `429` and `503`, responses include `Retry-After`.
