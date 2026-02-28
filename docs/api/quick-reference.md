# API Quick Reference

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: provide a one-page API cheat sheet with canonical request examples.

## Core requests

```bash
curl -fsS 'http://127.0.0.1:8080/v1/datasets'
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=grch38&limit=5'
curl -fsS 'http://127.0.0.1:8080/v1/genes/ENSG00000139618/transcripts?release=110&species=homo_sapiens&assembly=grch38'
```

## Request patterns

- List endpoints use `limit` and optional `cursor`.
- Use explicit `release`, `species`, and `assembly` for dataset-bound calls.
- Prefer documented `include` tokens only.

## Error handling

See [Errors](errors.md) and [Reference Errors](../reference/errors.md).

## Next

- [Pagination](pagination.md)
- [Reference](../reference/index.md)
