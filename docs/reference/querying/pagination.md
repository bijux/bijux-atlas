# Pagination

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define stable pagination semantics for query endpoints.

## Rules

- Cursor-based pagination is canonical.
- Cursor values are opaque and immutable.
- `limit` controls page size within configured bounds.
- Reusing a cursor with a different query shape is invalid.

## Example

First page:

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=2'
```

Next page:

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=2&cursor=<cursor>'
```

## Related

- [Query Parameters](query-parameters.md)
- [API Pagination Guide](../../api/pagination.md)
