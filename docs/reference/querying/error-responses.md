# Error Responses

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define canonical error codes and response envelope behavior for query surfaces.

## Envelope

Errors return a stable object with `code`, `message`, optional `details`, and `request_id`.

## Common query errors

- `InvalidQueryParameter`
- `MissingDatasetDimension`
- `InvalidCursor`
- `ValidationFailed`
- `UpstreamStoreUnavailable`

## Related

- [API Errors](../../api/errors.md)
- [Retry Logic](retry-logic.md)
