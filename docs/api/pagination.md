# Pagination

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: define one canonical pagination contract for list endpoints.

## Contract

- Cursor tokens are opaque.
- Cursor payload binds to dataset identity and sort contract.
- Invalid or mismatched cursors return `InvalidCursor`.

## Parameters

- `limit`: optional page size parameter.
- `cursor`: optional continuation token.

## Response Behavior

- `page.next_cursor` is set only when additional results exist.
- `links.next_cursor` mirrors `page.next_cursor`.
- Backward pagination is out of scope for v1.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
