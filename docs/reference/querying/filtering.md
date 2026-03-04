# Filtering

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define stable filtering semantics for query endpoints.

## Filtering model

- Filters are additive.
- Multiple filters combine with logical AND.
- Invalid filter values return contract-defined error responses.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&chromosome=13&biotype=protein_coding'
```

## Related

- [Query Parameters](query-parameters.md)
- [Errors](../errors.md)
