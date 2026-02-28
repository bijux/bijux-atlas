# API Quick Reference

Owner: `api-contracts`  
Type: `reference`  
Surface version: `v1`  
Reason to exist: provide one-page endpoint and contract navigation for API consumers.

## Core Endpoints

- `GET /v1/datasets`
- `GET /v1/datasets/{release}/{species}/{assembly}`
- `GET /v1/genes`
- `GET /v1/genes/{gene_id}/transcripts`
- `GET /v1/sequence/region`
- `GET /v1/diff/genes`
- `POST /v1/query/validate`

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/datasets'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
