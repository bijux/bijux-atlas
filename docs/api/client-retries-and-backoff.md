# Client retries and backoff

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define safe retry behavior for Atlas API clients.

## Policy

- Retry only idempotent reads and validation requests.
- Do not retry caller-owned failures such as `InvalidQueryParameter`, `MissingDatasetDimension`, or `InvalidCursor`.
- For `RateLimited`, `Timeout`, and `UpstreamStoreUnavailable`, use bounded exponential backoff with jitter.
- Stop retrying when the total request budget is exhausted or the server keeps returning the same failure class.

## Safe defaults

- Maximum attempts: `4`
- Base backoff: `120ms`
- Jitter: full jitter per attempt
- Concurrency: keep request fan-out bounded to avoid retry storms

## Verification

Use the retry policy together with:

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5'
```

Expected output: a successful `200` response without unbounded client retries, or a stable error code that lets the client stop safely.

## Next steps

- [Client behavior expectations](client-behavior.md)
- [Errors](errors.md)
