# API quick reference

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: provide a one-page API cheat sheet with canonical request examples.

## Core requests

```bash
curl -fsS 'http://127.0.0.1:8080/v1/datasets?limit=1'
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5'
curl -fsS 'http://127.0.0.1:8080/v1/genes/ENSG00000139618/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=5'
```

Expected output:

- `GET /v1/datasets` returns `api_version`, `contract_version`, and a `data.items` array.
- `GET /v1/genes` returns `data.rows` and may return `page.next_cursor`.
- `GET /v1/genes/{gene_id}/transcripts` returns transcript rows scoped to one dataset identity.

## Request patterns

- List endpoints use `limit` and optional `cursor`.
- Dataset-backed requests must include `release`, `species`, and `assembly`.
- Treat `next_cursor` as opaque.
- Use only documented `include` tokens and query parameters.

## Verification

Run the requests against a local or deployed Atlas endpoint. Success means the server returns HTTP `200` with a JSON body shaped as documented in [V1 surface](v1-surface.md).

## Next steps

- [Pagination](pagination.md)
- [Errors](errors.md)
- [Reference](../reference/index.md)
