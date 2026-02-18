# API Pagination

- Owner: `bijux-atlas-api`
- Stability: `stable`

## Cursor Contract

- Cursor tokens are opaque and HMAC integrity-checked.
- Cursor payload v1 binds to dataset identity and sort key.
- Invalid, version-mismatched, or dataset-mismatched cursors return `InvalidCursor` with one of:
  - `CURSOR_INVALID`
  - `CURSOR_VERSION_UNSUPPORTED`
  - `CURSOR_DATASET_MISMATCH`

## Parameters

- `limit` is the page-size control for v1.
- Default `limit` is `100`; max `limit` is `500`.
- `cursor` advances forward paging only.

## Semantics

- `page.next_cursor` is present when `has_more=true`; otherwise `null`.
- `links.next_cursor` mirrors `page.next_cursor`.
- `prev_cursor` is a non-goal for v1.
- Cursor compatibility policy: additive-only changes; v1 decoders remain backward compatible for v1 cursors.
