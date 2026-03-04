# Cursor Usage

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define how cursor tokens must be generated, consumed, and invalidated.

## Rules

- Cursors are opaque and request-shape bound.
- A cursor is valid only with the same endpoint and equivalent query filters.
- Cursor reuse across releases or dataset dimensions is invalid.
- Clients must restart pagination when receiving `InvalidCursor`.

## Related

- [Pagination](pagination.md)
- [Error Responses](error-responses.md)
