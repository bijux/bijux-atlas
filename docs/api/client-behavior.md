# Client behavior expectations

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define safe default client behavior for dataset identity, caching, retries, and idempotent use.

## Expectations

- Send explicit dataset identity for dataset-backed queries.
- Reuse cursor tokens only with the exact matching query shape.
- Honor response cache headers and `ETag` values when present.
- Keep retries bounded and jittered using [Client retries and backoff](client-retries-and-backoff.md).

## Verification

```bash
curl -fsS 'http://127.0.0.1:8080/v1/query/validate' \
  -H 'content-type: application/json' \
  -d '{"release":"110","species":"homo_sapiens","assembly":"GRCh38","gene_id":"ENSG00000139618"}'
```

Expected output: HTTP `200` with a JSON body that includes `data.query_class`, `data.dataset`, and `data.limits`.

## Next steps

- [Client retries and backoff](client-retries-and-backoff.md)
- [Performance expectations](performance-expectations.md)
