# Pagination

- Owner: `bijux-atlas-query`

Cursor pagination uses signed opaque cursors bound to normalized query parameters.

- Stable ordering is mandatory for cursor correctness.
- Cursor decoding is backward compatible within v1.
- Invalid signatures return `InvalidCursor`.
