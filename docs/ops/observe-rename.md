> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

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

Migration completed on 2026-02-25. Contracts, inventories, schemas, and docs use `ops/observe/` only.
