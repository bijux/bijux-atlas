# Retry Logic

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define safe retry behavior for transient query failures.

## Rules

- Retry idempotent reads only.
- Never retry caller-owned validation errors.
- Use bounded exponential backoff with jitter for transient failures.
- Stop after request budget is exhausted.

## Related

- [Error Responses](error-responses.md)
- [Client Retries and Backoff](../../api/client-retries-and-backoff.md)
