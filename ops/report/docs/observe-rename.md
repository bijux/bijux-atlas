# Observability Directory Rename

- Owner: bijux-atlas-operations
- Stability: stable

## Decision

Canonical observability directory name is `ops/observe/`.

## Compatibility Window

- Legacy compatibility path: none.
- New references must use `ops/observe/`.
- Compatibility ended on 2026-02-25.

## Cutover Rule

Legacy `ops/observe/` is removed. Contracts and schemas must use `ops/observe/` only.
