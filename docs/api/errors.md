# Errors

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: explain user-facing error behavior and remediation flow.

## Error envelope

Responses return a stable `error` object with `code`, `message`, optional `details`, and `request_id`.

## How to interpret failures

1. Inspect `error.code` first.
2. Check whether the failure is caller-owned, retryable, or service-owned.
3. Only retry when the error is transient and your request is safe to replay.
4. Escalate persistent service-owned failures with [Operations incident response](../operations/incident-response.md).

## Common interpretations

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/genes?limit=0'
```

- `InvalidQueryParameter` or `ValidationFailed`: the request shape is wrong. Fix inputs before retrying.
- `MissingDatasetDimension`: add the missing `release`, `species`, or `assembly`.
- `InvalidCursor`: restart from the first page or use the matching cursor/query pair.
- `RateLimited`, `Timeout`, or `UpstreamStoreUnavailable`: follow [Client retries and backoff](client-retries-and-backoff.md).
- `NotReady` or repeated `Internal`: treat as a service issue and involve operators.

Expected output: the server returns a non-`200` response containing an `error` object with a stable `code`.

## Canonical code list

The taxonomy and status map live in [Reference errors](../reference/errors.md).

## Next steps

- [Compatibility](compatibility.md)
- [Troubleshooting](troubleshooting.md)
- [Reference errors](../reference/errors.md)
