# Pagination

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define pagination semantics with runnable examples.

## Semantics

- Cursor tokens are opaque and must be treated as immutable.
- `limit` bounds response size for list endpoints.
- `next_cursor` appears only when more results are available.

## Example

First page:

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=grch38&limit=2'
```

Next page (replace `<cursor>`):

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=grch38&limit=2&cursor=<cursor>'
```

## Failure behavior

Invalid or mismatched cursor values return the documented cursor error code.

## Next

- [Errors](errors.md)
- [Reference Schemas](../reference/schemas.md)
