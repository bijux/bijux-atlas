# Pagination

Owner: `api-contracts`  
Type: `guide`  
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

## Related Pages

- [API](index.md)
- [Versioning Policy](versioning.md)
