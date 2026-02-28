# Errors

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: explain user-facing error behavior and remediation flow.

## Error envelope

Responses return a stable `error` object with `code`, `message`, optional `details`, and `request_id`.

## What to do when a request fails

1. Inspect `error.code` and request parameters.
2. Retry only when the code is retryable by policy.
3. Escalate persistent service errors using [Operations Incident Response](../operations/incident-response.md).

## Examples

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/genes?limit=0'
```

## Canonical code list

See [Reference Errors](../reference/errors.md).

## Next

- [Compatibility](compatibility.md)
- [Reference Errors](../reference/errors.md)
