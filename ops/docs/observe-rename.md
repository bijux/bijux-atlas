# Observability Directory Rename

- Owner: bijux-atlas-operations
- Stability: stable

## Decision

Canonical observability directory name is `ops/observe/`.

## Compatibility Window

- Legacy compatibility path: `ops/obs/`
- New references must use `ops/observe/`.
- Existing references under compatibility scope are tracked and reduced incrementally.

## Cutover Rule

After compatibility cutoff, `ops/obs/` is removed and all contracts point to `ops/observe/`.
