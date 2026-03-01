# Redirects Policy

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define stable redirect behavior for documentation moves and removals.

## Redirect Contract

- Every moved or renamed stable page must add an entry to `docs/redirects.json`.
- Redirect keys use old repository-relative doc paths.
- Redirect values use current canonical paths.
- Redirects stay in place until all known inbound references are updated.

## Required Change Checklist

- Add mapping in `docs/redirects.json`.
- Update section indexes to point to canonical location.
- Remove direct links to the old path from reader pages.
- Note the move in the commit message.

## Validation

Run docs contract checks to ensure no broken inbound reader links remain.
