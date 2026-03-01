# Performance expectations

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: set practical expectations for API clients around latency, payload size, and overload behavior.

## What to expect

- Cheap catalog and small query requests should complete quickly enough for interactive use.
- Large ranges, broad filters, and high `limit` values can be rejected instead of running indefinitely.
- Overload protection favors predictable failure over tail-latency collapse.

## Client guidance

- Keep `limit` values modest unless you have measured a larger need.
- Use pagination for bulk reads instead of oversized single requests.
- Use `POST /v1/query/validate` to classify expensive requests before issuing them at scale.
- Treat `429`, `422`, and `413` responses as signals to reduce query cost.

## Verification

```bash
curl -fsS 'http://127.0.0.1:8080/v1/query/validate' \
  -H 'content-type: application/json' \
  -d '{"release":"110","species":"homo_sapiens","assembly":"GRCh38","range":"1:1-1000","limit":25}'
```

Expected output: HTTP `200` with a query classification and limit guidance that can be used before issuing the full request.

## Next steps

- [Client behavior expectations](client-behavior.md)
- [Troubleshooting](troubleshooting.md)
