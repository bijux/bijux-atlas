# Query Command Example

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: provide a canonical query command for validating response semantics.

## Example command

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&symbol=BRCA2&limit=20'
```

## Expected result

A valid JSON response that includes deterministic ordering and pagination metadata.

## Related

- [Query Parameters](../querying/query-parameters.md)
- [Pagination Semantics](../querying/pagination.md)
