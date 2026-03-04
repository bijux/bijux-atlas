# Projections

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define stable projection semantics for selecting response fields.

## Projection model

- `fields` controls the returned data columns.
- Field names must be valid for the endpoint schema.
- Field order in response follows canonical response model.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&fields=gene_id,symbol,chromosome'
```

## Related

- [Query Parameters](query-parameters.md)
- [API Response Examples](../examples/api-response-examples.md)
