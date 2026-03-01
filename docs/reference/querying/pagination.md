# Pagination semantics

Owner: `bijux-atlas-api`
Type: `reference`
Reason to exist: define the canonical pagination contract for API readers and operators.

## Contract

- Cursor-based pagination is the canonical default.
- Clients must treat cursors as opaque tokens.
- Default and maximum limits are defined by the API surface contract.

## See also

- `docs/api/index.md`
- `docs/reference/index.md`
