# V1 Surface

- Owner: `api-contracts`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define the stable endpoint surface for v1 clients.

## Endpoints

- `GET /healthz`
- `GET /readyz`
- `GET /metrics`
- `GET /v1/datasets`
- `GET /v1/datasets/{release}/{species}/{assembly}`
- `GET /v1/genes`
- `GET /v1/genes/count`
- `GET /v1/genes/{gene_id}/transcripts`
- `GET /v1/genes/{gene_id}/sequence`
- `GET /v1/sequence/region`
- `GET /v1/diff/genes`
- `GET /v1/diff/region`
- `POST /v1/query/validate`
- `GET /v1/version`
- `GET /v1/openapi.json`

## Verification

```bash
curl -fsS 'http://127.0.0.1:8080/v1/version'
```

Expected output: HTTP `200` with version metadata for the running service and published plugin set.

## Next steps

- [Reference schemas](../reference/schemas.md)
- [Errors](errors.md)
