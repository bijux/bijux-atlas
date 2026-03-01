# Pagination

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: show API consumers how to use the stable pagination contract safely.

## How to use pagination

- Cursor tokens are opaque and must be treated as immutable.
- `limit` bounds response size for list endpoints.
- `next_cursor` appears only when more results are available.

## Example

First page:

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=2'
```

Next page (replace `<cursor>`):

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=2&cursor=<cursor>'
```

Expected output:

- The first response may contain `page.next_cursor`.
- The follow-up request returns the next slice of rows for the same query contract.
- A changed query shape with the same cursor must fail instead of returning mixed pages.

## Semantics reference

The canonical cursor contract lives in [Reference querying pagination](../reference/querying/pagination.md) and [Reference schemas](../reference/schemas.md).

## Failure behavior

Invalid or mismatched cursor values return `InvalidCursor`. Missing dataset dimensions return `MissingDatasetDimension`.

## Next steps

- [Errors](errors.md)
- [Reference querying pagination](../reference/querying/pagination.md)
